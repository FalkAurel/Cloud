use sqlx::Row;
use uuid::Uuid;

use crate::database::ReadOnly;

const GET_FILE_QUERY: &str = r#"
SELECT id, name
FROM files
WHERE id = ? AND user_id = ? AND is_folder = FALSE;
"#;

pub(crate) struct FileEntry {
    pub id: Uuid,
    pub name: String,
}

pub(crate) struct GetFile {
    file_id: Uuid,
    user_id: i32,
}

impl GetFile {
    pub fn new(file_id: Uuid, user_id: i32) -> Self {
        Self { file_id, user_id }
    }
}

impl ReadOnly for GetFile {
    type Success = Option<FileEntry>;
    type Error = sqlx::Error;

    async fn read(&self, pool: &sqlx::Pool<sqlx::MySql>) -> Result<Self::Success, Self::Error> {
        sqlx::query(GET_FILE_QUERY)
            .bind(self.file_id)
            .bind(self.user_id)
            .fetch_optional(pool)
            .await
            .map(|opt| {
                opt.map(|row| FileEntry {
                    id: row.get("id"),
                    name: row.get("name"),
                })
            })
    }
}
