use crate::{data_definitions::Auth, object_storage::Storage};
use rocket::{
    Data, State,
    data::{DataStream, ToByteUnit},
    http::{ContentType, MediaType, Status},
    post,
};
use sqlx::{MySql, Pool};
use tracing::{error, info, warn};

#[post("/upload", data = "<data>")]
pub async fn upload(
    auth: Auth,
    content_type: &ContentType,
    data: Data<'_>,
    db: &State<Pool<MySql>>,
    storage: &State<Box<dyn Storage>>,
) -> Result<Status, (Status, &'static str)> {
    if *content_type.media_type() != MediaType::Binary {
        warn!(
            user_id = %auth.get_jwt().user_id,
            content_type = %content_type,
            "Rejected upload: invalid content type"
        );

        return Err((
            Status::BadRequest,
            "Content-Type must be application/octet-stream",
        ));
    }
    let mut stream: DataStream = data.open(400.mebibytes());

    match storage.store(&mut stream).await {
        Ok(object_identifier) => {
            info!(user_id = auth.get_jwt().user_id, "Upload successful");

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
