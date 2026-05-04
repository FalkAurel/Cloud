use chrono::{DateTime, Utc};
use rocket::{State, get, http::Status, serde::json::Json};
use serde::Serialize;
use sqlx::{MySql, Pool};
use tracing::{error, info, instrument};

use crate::{
    ObjectID,
    data_definitions::{Auth, id::ID},
    database::{ReadOnly, virtual_filesystem::VirtualFileSystem},
};

#[derive(Serialize)]
pub struct FileResponse {
    id: ID,
    name: String,
    size_bytes: u64,
    is_folder: bool,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
}

#[instrument(skip(db))]
#[get("/files?<parent_id>")]
pub async fn list_files(
    auth: Auth,
    parent_id: Option<&str>,
    db: &State<Pool<MySql>>,
) -> Result<(Status, Json<Vec<FileResponse>>), (Status, &'static str)> {
    let user_id: ID = auth.get_jwt().user_id;

    let parent_id: Option<ObjectID> = match parent_id {
        Some(str) => match ObjectID::parse_str(str) {
            Ok(uuid) => Some(uuid),
            Err(_) => return Err((Status::BadRequest, "Invalid parent_id UUID")),
        },
        None => None,
    };

    if let Some(ref parent_id) = parent_id {
        info!(user_id=%user_id, parent_id=%parent_id.0, "Listing files");
    }

    match VirtualFileSystem::list_files(user_id, parent_id)
        .read(db)
        .await
    {
        Ok(rows) => {
            let response: Vec<FileResponse> = rows
                .into_iter()
                .map(|row| FileResponse {
                    id: row.id,
                    name: row.name,
                    size_bytes: row.size_bytes,
                    is_folder: row.is_folder,
                    created_at: row.created_at,
                    modified_at: row.modified_at,
                })
                .collect();
            Ok((Status::Ok, Json(response)))
        }
        Err(e) => {
            error!(user_id=%user_id, error = %e, "Failed to list files");
            Err((Status::InternalServerError, "Internal server error"))
        }
    }
}
