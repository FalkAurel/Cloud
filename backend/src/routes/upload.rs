use crate::ObjectID;
use crate::data_definitions::JWT;
use crate::database::Transactional;
use crate::{
    data_definitions::Auth, database::virtual_filesystem::VirtualFileSystem,
    object_storage::Storage,
};
use rocket::Request;
use rocket::serde::json::Json;
use rocket::{
    Data, State,
    data::{DataStream, ToByteUnit},
    http::{MediaType, Status},
    post,
    request::{FromRequest, Outcome},
};

use serde::Serialize;
use sqlx::{MySql, Pool, Transaction};
use tracing::{error, info, instrument, warn};
use uuid::Uuid;

#[derive(Serialize)]
pub struct UploadResponse {
    id: Uuid,
}



#[derive(Serialize)]
pub struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug)]
pub struct FileMetaData<'a> {
    pub(crate) size: u64,
    pub(crate) name: &'a str,
    pub(crate) is_folder: bool,
    pub(crate) parent_id: Option<Uuid>,
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

            let parent_id: Option<Uuid> = match request.headers().get_one("X-ParentUuid") {
                Some(val) => match Uuid::parse_str(val) {
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
                size,
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
    meta_data: FileMetaData<'_>,
    data: Data<'_>,
    db: &State<Pool<MySql>>,
    storage: &State<Box<dyn Storage>>,
) -> Result<(Status, Json<UploadResponse>), (Status, Json<ErrorResponse>)> {
    let jwt: JWT = auth.get_jwt();
    info!(user_id = jwt.user_id, name = meta_data.name, is_folder = meta_data.is_folder, "Upload started");

    let mut transaction: Transaction<MySql> = match db.begin().await {
        Ok(tx) => tx,
        Err(err) => {
            error!(user_id = jwt.user_id, error = %err, "Failed to begin transaction");
            return Err((Status::InternalServerError, Json(ErrorResponse { error: "Failed to begin transaction" })));
        }
    };

    let object_id: ObjectID = if meta_data.is_folder {
        match create_folder(&jwt, &meta_data, &mut transaction).await {
            Ok(id) => id,
            Err(err) => {
                error!(user_id = jwt.user_id, name = meta_data.name, error = %err, "Failed to create folder");
                let _ = transaction.rollback().await;
                return Err((Status::InternalServerError, Json(ErrorResponse { error: "Failed to create folder" })));
            }
        }
    } else {
        let mut stream: DataStream = data.open(200.mebibytes());
        let object_identifier: ObjectID = match storage.store(&mut stream).await {
            Ok(id) => id,
            Err(err) => {
                error!(user_id = jwt.user_id, name = meta_data.name, error = %err, "Failed to store object");
                return Err((Status::InternalServerError, Json(ErrorResponse { error: "Failed to store file" })));
            }
        };
        info!(user_id = jwt.user_id, object_id = %object_identifier.0, "Object stored");

        match create_file(object_identifier, &jwt, &meta_data, &mut transaction).await {
            Ok(id) => id,
            Err(err) => {
                error!(user_id = jwt.user_id, name = meta_data.name, error = %err, "Failed to write file metadata");
                let _ = transaction.rollback().await;
                if let Err(del_err) = storage.delete(object_identifier).await {
                    warn!(user_id = jwt.user_id, object_id = %object_identifier.0, error = %del_err, "Failed to delete orphaned object");
                }
                return Err((Status::InternalServerError, Json(ErrorResponse { error: "Failed to save file metadata" })));
            }
        }
    };

    if let Err(err) = transaction.commit().await {
        error!(user_id = jwt.user_id, error = %err, "Failed to commit transaction");
        if !meta_data.is_folder {
            if let Err(del_err) = storage.delete(object_id).await {
                warn!(user_id = jwt.user_id, object_id = %object_id.0, error = %del_err, "Failed to delete orphaned object after commit failure");
            }
        }
        return Err((Status::InternalServerError, Json(ErrorResponse { error: "Failed to commit transaction" })));
    }

    info!(user_id = jwt.user_id, object_id = %object_id.0, name = meta_data.name, "Upload complete");
    Ok((Status::Created, Json(UploadResponse { id: object_id.0 })))
}

async fn create_folder(
    jwt: &JWT,
    meta_data: &FileMetaData<'_>,
    transaction: &mut Transaction<'_, MySql>,
) -> Result<ObjectID, sqlx::Error> {
    let object_uuid: Uuid = Uuid::new_v4();
    VirtualFileSystem::create_folder(
        object_uuid,
        jwt.user_id,
        meta_data.name,
        meta_data.parent_id,
    )
    .execute(transaction)
    .await?;

    Ok(ObjectID(object_uuid))
}

async fn create_file(
    object_identifier: ObjectID,
    jwt: &JWT,
    meta_data: &FileMetaData<'_>,
    transaction: &mut Transaction<'_, MySql>
) -> Result<ObjectID, sqlx::Error> {

    VirtualFileSystem::create_file(
        object_identifier.0, 
        jwt.user_id, 
        meta_data.name, 
        meta_data.size, 
        meta_data.parent_id
    ).execute(transaction).await?;

    Ok(object_identifier)
}

#[cfg(test)]
mod tests {
    use rocket::Rocket;
    use rocket::http::{ContentType, Cookie, Header, Status as HttpStatus};
    use rocket::local::asynchronous::Client;
    use rocket::routes;
    use sqlx::{MySql, Pool};

    use super::upload;
    use crate::TOKEN_LIFETIME;
    use crate::data_definitions::JWT;
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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
        let client = build_client::<true>().await;
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        let email = "upload_file_test@example.com";
        let token = create_test_user(db, email).await;

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
        let client = build_client::<true>().await;
        let db = client.rocket().state::<Pool<MySql>>().unwrap();
        let email = "upload_folder_test@example.com";
        let token = create_test_user(db, email).await;

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
        let client = build_client::<false>().await;
        let token = JWT::create(1, TOKEN_LIFETIME).unwrap();

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
