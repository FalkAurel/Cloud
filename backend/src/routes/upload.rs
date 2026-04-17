use crate::data_definitions::JWT;
use crate::database::Transactional;
use crate::{
    data_definitions::Auth, database::virtual_filesystem::VirtualFileSystem,
    object_storage::Storage,
};
use rocket::Request;
use rocket::{
    Data, State,
    data::{DataStream, ToByteUnit},
    http::{MediaType, Status},
    post,
    request::{FromRequest, Outcome},
};
use sqlx::{MySql, Pool, Transaction};
use tracing::{error, info};
pub struct FileMetaData<'a> {
    pub(crate) size: u64,
    pub(crate) name: &'a str,
    pub(crate) is_folder: bool,
}

#[derive(Debug)]
pub enum FileMetaError {
    MissingContentLength,
    InvalidContentLength,
    MissingFilename,
    MissingXFolder,
    InvalidXFolderValue,
    InvalidContentType,
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
            })
        })
    }
}

#[post("/upload", data = "<data>")]
pub async fn upload(
    auth: Auth,
    meta_data: FileMetaData<'_>,
    data: Data<'_>,
    db: &State<Pool<MySql>>,
    storage: &State<Box<dyn Storage>>,
) -> Result<Status, (Status, &'static str)> {
    let mut stream: DataStream = data.open(400.mebibytes());

    match storage.store(&mut stream).await {
        Ok(object_identifier) => {
            let jwt: JWT = auth.get_jwt();
            info!(user_id = jwt.user_id, "Upload successful");

            let create_file = VirtualFileSystem::create_file(
                object_identifier.0,
                jwt.user_id,
                meta_data.name,
                meta_data.size,
                None, // How to load this shit from the user request
                meta_data.is_folder,
            );

            let mut transaction: Transaction<MySql> = match db.begin().await {
                Ok(tx) => tx,
                Err(err) => {
                    error!(user_id = jwt.user_id, error = %err, "Failed to begin transaction");
                    if let Err(del_err) = storage.delete(object_identifier).await {
                        error!(user_id = jwt.user_id, error = %del_err, "Failed to delete orphaned MinIO object");
                    }
                    return Err((Status::InternalServerError, "Failed to save file metadata"));
                }
            };

            if let Err(err) = create_file.execute(&mut transaction).await {
                error!(user_id = jwt.user_id, error = %err, "Failed to insert file metadata");

                if let Err(rb_err) = transaction.rollback().await {
                    error!(user_id = jwt.user_id, error = %rb_err, "Rollback failed");
                }

                if let Err(del_err) = storage.delete(object_identifier).await {
                    error!(user_id = jwt.user_id, error = %del_err, "Failed to delete orphaned MinIO object");
                }

                return Err((Status::InternalServerError, "Failed to save file metadata"));
            }

            if let Err(err) = transaction.commit().await {
                error!(user_id = jwt.user_id, error = %err, "Failed to commit transaction");
                if let Err(del_err) = storage.delete(object_identifier).await {
                    error!(user_id = jwt.user_id, error = %del_err, "Failed to delete orphaned MinIO object after commit failure");
                }
                return Err((Status::InternalServerError, "Failed to save file metadata"));
            }

            info!(
                user_id = jwt.user_id,
                "File metadata persisted successfully"
            );
            Ok(Status::Ok)
        }
        Err(e) => {
            error!(
                user_id = auth.get_jwt().user_id,
                error = %e,
                "Upload failed during storage"
            );

            Err((Status::InternalServerError, "Failed to store uploaded file"))
        }
    }
}
