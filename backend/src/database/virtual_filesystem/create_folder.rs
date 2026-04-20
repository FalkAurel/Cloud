use uuid::Uuid;

use crate::database::Transactional;

const CREATE_FOLDER_QUERY: &str = r#"
INSERT INTO files (id, user_id, name, size_bytes, parent_id, is_folder) VALUES (?, ?, ?, ?, ?, ?);
"#;

pub(crate) struct CreateFolder<'a> {
    id: Uuid,
    user_id: i32,
    name: &'a str,
    size_bytes: u64,
    parent_id: Option<Uuid>,
    is_folder: bool,
}

impl<'a> CreateFolder<'a> {
    pub(crate) fn new(id: Uuid, user_id: i32, name: &'a str, parent_id: Option<Uuid>) -> Self {
        Self {
            id,
            user_id,
            name,
            size_bytes: 0,
            parent_id,
            is_folder: true,
        }
    }
}

impl<'a> Transactional for CreateFolder<'a> {
    type Success = ();
    type Error = sqlx::Error;
    fn execute<'t>(
        &self,
        tx: &'t mut sqlx::Transaction<'_, sqlx::MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send {
        async {
            sqlx::query(CREATE_FOLDER_QUERY)
                .bind(self.id)
                .bind(self.user_id)
                .bind(self.name)
                .bind(self.size_bytes)
                .bind(self.parent_id)
                .bind(self.is_folder)
                .execute(&mut **tx)
                .await?;

            Ok(())
        }
    }
}
