use chrono::{DateTime, Utc};
use sqlx::Row;
use uuid::Uuid;

use crate::database::ReadOnly;

const LIST_FILES_QUERY: &str = r#"
SELECT id, name, size_bytes, is_folder, created_at, modified_at
FROM files
WHERE user_id = ?
  AND (parent_id <=> ?)
ORDER BY is_folder DESC, name ASC;
"#;

pub(crate) struct FileRow {
    pub id: Uuid,
    pub name: String,
    pub size_bytes: u64,
    pub is_folder: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub(crate) struct ListFiles {
    user_id: i32,
    parent_id: Option<Uuid>,
}

impl ListFiles {
    pub fn new(user_id: i32, parent_id: Option<Uuid>) -> Self {
        Self { user_id, parent_id }
    }
}

impl ReadOnly for ListFiles {
    type Success = Vec<FileRow>;
    type Error = sqlx::Error;

    async fn read(&self, pool: &sqlx::Pool<sqlx::MySql>) -> Result<Self::Success, Self::Error> {
        let rows = sqlx::query(LIST_FILES_QUERY)
            .bind(self.user_id)
            .bind(self.parent_id)
            .fetch_all(pool)
            .await?;

        Ok(rows
            .into_iter()
            .map(|row| FileRow {
                id: row.get("id"),
                name: row.get("name"),
                size_bytes: row.get::<u64, _>("size_bytes"),
                is_folder: row.get("is_folder"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
            })
            .collect())
    }
}
