use std::num::NonZero;
use std::sync::Arc;

use crate::ObjectID;
use crate::data_definitions::JWT;
use crate::data_definitions::file::{Directory, EntryDraft, File, State as FileState};
use crate::database::Transactional;
use crate::{
    data_definitions::Auth, database::virtual_filesystem::VirtualFileSystem,
    object_storage::Storage,
};
use rocket::data::{ByteUnit, DataStream};
use rocket::serde::json::Json;
use rocket::{
    Data,
    data::ToByteUnit,
    http::{MediaType, Status},
    post,
    request::{FromRequest, Outcome},
};
use rocket::{Request, State};

use serde::Serialize;
use sqlx::{MySql, Pool, Transaction};
use tracing::{info, instrument};

#[derive(Serialize)]
pub struct UploadResponse {
    id: ObjectID,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug)]
pub struct FileMetaData<'a> {
    pub(crate) size: Option<NonZero<u64>>,
    pub(crate) name: &'a str,
    pub(crate) is_folder: bool,
    pub(crate) parent_id: Option<ObjectID>,
}

#[derive(Debug)]
pub enum FileMetaError {
    MissingContentLength,
    InvalidContentLength,
    MissingFilename,
    MissingXFolder,
    InvalidXFolderValue,
    InvalidContentType,
    InvalidCombination,
    InvalidParentUuid,
}

impl<'a> FromRequest<'a> for FileMetaData<'a> {
    type Error = FileMetaError;

    fn from_request<'life0, 'async_trait>(
        request: &'a Request<'life0>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<Output = Outcome<Self, Self::Error>>
                + ::core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'a: 'async_trait,
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            // Content-Length
            let size_header: &str = match request.headers().get_one("Content-Length") {
                Some(val) => val,
                None => {
                    return Outcome::Error((
                        Status::LengthRequired,
                        FileMetaError::MissingContentLength,
                    ));
                }
            };

            let size: u64 = match size_header.parse::<u64>() {
                Ok(s) => s,
                Err(_) => {
                    return Outcome::Error((
                        Status::BadRequest,
                        FileMetaError::InvalidContentLength,
                    ));
                }
            };

            // Filename
            let name: &str = match request.headers().get_one("X-Filename") {
                Some(name) => name,
                None => {
                    return Outcome::Error((Status::BadRequest, FileMetaError::MissingFilename));
                }
            };

            // IsFolder
            let is_folder: bool = match request.headers().get_one("X-IsFolder") {
                Some(value) => match value {
                    "true" | "1" => true,
                    "false" | "0" => false,
                    _ => {
                        return Outcome::Error((
                            Status::BadRequest,
                            FileMetaError::InvalidXFolderValue,
                        ));
                    }
                },
                None => return Outcome::Error((Status::BadRequest, FileMetaError::MissingXFolder)),
            };

            if is_folder && size > 0 {
                return Outcome::Error((Status::BadRequest, FileMetaError::InvalidCombination));
            }

            let parent_id: Option<ObjectID> = match request.headers().get_one("X-ParentUuid") {
                Some(val) => match ObjectID::parse_str(val) {
                    Ok(uuid) => Some(uuid),
                    Err(_) => {
                        return Outcome::Error((
                            Status::BadRequest,
                            FileMetaError::InvalidParentUuid,
                        ));
                    }
                },
                None => None,
            };

            // Content-Type
            if !request
                .content_type()
                .is_some_and(|ct| *ct.media_type() == MediaType::Binary)
            {
                return Outcome::Error((
                    Status::UnsupportedMediaType,
                    FileMetaError::InvalidContentType,
                ));
            }

            Outcome::Success(FileMetaData {
                size: NonZero::new(size),
                name,
                is_folder,
                parent_id,
            })
        })
    }
}

#[instrument(skip(data, db, storage))]
#[post("/upload", data = "<data>")]
pub async fn upload(
    auth: Auth,
    db: &State<Pool<MySql>>,
    storage: &State<Arc<dyn Storage>>,
    meta_data: FileMetaData<'_>,
    data: Data<'_>,
) -> Result<(Status, Json<UploadResponse>), (Status, Json<ErrorResponse>)> {
    let jwt: JWT = auth.get_jwt();
    info!(
        user_id = %jwt.user_id,
        name = meta_data.name,
        is_folder = meta_data.is_folder,
        "Upload started"
    );

    let mut transaction: Transaction<MySql> = match db.inner().begin().await {
        Ok(transaction) => transaction,
        Err(_) => {
            return Err((
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "Database error",
                }),
            ));
        }
    };

    let object_id: ObjectID = if meta_data.is_folder {
        create_folder(&jwt, &meta_data, &mut transaction)
            .await
            .map_err(|_| {
                (
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error: "Failed to create folder",
                    }),
                )
            })?
    } else {
        let id: ObjectID = create_file(&jwt, &meta_data, &mut transaction)
            .await
            .map_err(|_| {
                (
                    Status::InternalServerError,
                    Json(ErrorResponse {
                        error: "Failed to create file record",
                    }),
                )
            })?;

        let limit: ByteUnit = meta_data.size.map(|s| s.get()).unwrap_or(0).bytes();
        let mut stream: DataStream = data.open(limit);
        // This is an implicit return -> Dropping the transaction -> Implicit call
        storage.inner().store(id, &mut stream).await.map_err(|_| {
            (
                Status::InternalServerError,
                Json(ErrorResponse {
                    error: "Failed to store file",
                }),
            )
        })?;
        id
    };

    transaction.commit().await.map_err(|_| {
        (
            Status::InternalServerError,
            Json(ErrorResponse {
                error: "Transaction commit failed",
            }),
        )
    })?;

    Ok((Status::Created, Json(UploadResponse { id: object_id })))
}

async fn create_folder(
    jwt: &JWT,
    meta_data: &FileMetaData<'_>,
    transaction: &mut Transaction<'_, MySql>,
) -> Result<ObjectID, sqlx::Error> {
    let folder_entry: EntryDraft<Directory> = EntryDraft::new(
        jwt.user_id,
        meta_data.name,
        meta_data.size,
        meta_data.parent_id,
        FileState::Created,
    );

    let id: ObjectID = VirtualFileSystem::create_folder(&folder_entry)
        .execute(transaction)
        .await?;

    Ok(id)
}

async fn create_file(
    jwt: &JWT,
    meta_data: &FileMetaData<'_>,
    transaction: &mut Transaction<'_, MySql>,
) -> Result<ObjectID, sqlx::Error> {
    let file_entry: EntryDraft<'_, File> = EntryDraft::new(
        jwt.user_id,
        meta_data.name,
        meta_data.size,
        meta_data.parent_id,
        FileState::Pending,
    );

    VirtualFileSystem::create_file(&file_entry)
        .execute(transaction)
        .await
}

#[cfg(test)]
mod tests {
    use std::num::NonZero;

    use rocket::Rocket;
    use rocket::http::{ContentType, Cookie, Header, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::routes;
    use sqlx::{MySql, Pool};

    use super::upload;
    use crate::TOKEN_LIFETIME;
    use crate::data_definitions::JWT;
    use crate::data_definitions::id::ID;
    use crate::database::ReadOnly;
    use crate::database::user_repository::UserRepository;
    use crate::init_db;
    use crate::object_storage::mock_storage::MockStorage;
    use crate::test_harness_setup::cleanup_user_by_email;

    async fn build_client<const SUCCESS: bool>() -> Client {
        let storage: Box<dyn crate::object_storage::Storage> = Box::new(MockStorage::<SUCCESS>);
        let rocket = Rocket::build()
            .mount("/", routes![upload])
            .manage(init_db().await)
            .manage(storage);
        Client::tracked(rocket).await.unwrap()
    }

    // Inserts a user directly via SQL — avoids the email-feature guard on the signup route.
    async fn create_test_user(pool: &Pool<MySql>, email: &str) -> String {
        sqlx::query("INSERT INTO users (name, email, password) VALUES (?, ?, ?)")
            .bind("Upload Test")
            .bind(email)
            .bind("$argon2id$v=19$m=19456,t=2,p=1$c29tZXJhbmRvbXNhbHQ$RoB4RWBSupGkPkOKA7HiYRmFjhSeop6UVKzSFbGMFG4")
            .execute(pool)
            .await
            .unwrap();
        let id = UserRepository::get_login_view(email)
            .read(pool)
            .await
            .unwrap()
            .unwrap()
            .id;
        JWT::create(id, TOKEN_LIFETIME).unwrap()
    }

    // --- FileMetaData extractor tests (no DB/storage needed, no auth) ---

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_401_without_jwt() {
        let client = build_client::<true>().await;

        let response = client
            .post("/upload")
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "test.txt"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Unauthorized);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_411_without_content_length() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("X-Filename", "test.txt"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::LengthRequired);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_400_without_filename() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_400_without_x_is_folder() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "test.txt"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_400_for_invalid_x_is_folder_value() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "test.txt"))
            .header(Header::new("X-IsFolder", "yes"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_415_for_wrong_content_type() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "test.txt"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::JSON)
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::UnsupportedMediaType);
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_400_for_folder_with_nonzero_size() {
        let client: Client = build_client::<true>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "10"))
            .header(Header::new("X-Filename", "myfolder"))
            .header(Header::new("X-IsFolder", "true"))
            .header(ContentType::new("application", "octet-stream"))
            .body("")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::BadRequest);
    }

    // --- Storage / DB integration tests ---

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var and database"]
    async fn upload_returns_200_for_valid_file_request() {
        let client: Client = build_client::<true>().await;
        let db: &Pool<MySql> = client.rocket().state::<Pool<MySql>>().unwrap();
        let email: &str = "upload_file_test@example.com";
        let token: String = create_test_user(db, email).await;

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "hello.txt"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Created);
        cleanup_user_by_email(db, email).await;
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var and database"]
    async fn upload_returns_200_for_valid_folder_request() {
        let client: Client = build_client::<true>().await;
        let db: &Pool<MySql> = client.rocket().state::<Pool<MySql>>().unwrap();
        let email: &str = "upload_folder_test@example.com";
        let token: String = create_test_user(db, email).await;

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "0"))
            .header(Header::new("X-Filename", "mydir"))
            .header(Header::new("X-IsFolder", "true"))
            .header(ContentType::new("application", "octet-stream"))
            .body("")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::Created);
        cleanup_user_by_email(db, email).await;
    }

    #[tokio::test]
    #[ignore = "requires JWT_SECRET env var"]
    async fn upload_returns_500_when_storage_fails() {
        let client: Client = build_client::<false>().await;
        let token: String = JWT::create(ID(NonZero::new(1).unwrap()), TOKEN_LIFETIME).unwrap();

        let response = client
            .post("/upload")
            .cookie(Cookie::new("jwt", token))
            .header(Header::new("Content-Length", "4"))
            .header(Header::new("X-Filename", "hello.txt"))
            .header(Header::new("X-IsFolder", "false"))
            .header(ContentType::new("application", "octet-stream"))
            .body("data")
            .dispatch()
            .await;

        assert_eq!(response.status(), HttpStatus::InternalServerError);
    }
}
