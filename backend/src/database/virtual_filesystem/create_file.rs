use crate::database::Transactional;
use sqlx::error::Error;
use uuid::Uuid;

const CREATE_FILE_QUERY: &str = r#"
INSERT INTO files (id, user_id, name, size_bytes, parent_id, is_folder) VALUES (?, ?, ?, ?, ?, ?);
"#;

pub(crate) struct CreateFile<'a> {
    id: Uuid,
    user_id: i32,
    name: &'a str,
    size_bytes: u64,
    parent_id: Option<Uuid>,
    is_folder: bool,
}

impl<'a> CreateFile<'a> {
    pub fn new(
        id: Uuid,
        user_id: i32,
        name: &'a str,
        size_bytes: u64,
        parent_id: Option<Uuid>,
        is_folder: bool,
    ) -> Self {
        Self {
            id,
            user_id,
            name,
            size_bytes,
            parent_id,
            is_folder,
        }
    }
}

impl<'a> Transactional for CreateFile<'a> {
    type Success = ();
    type Error = Error;

    fn execute<'t>(
        &self,
        tx: &'t mut sqlx::Transaction<'_, sqlx::MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send {
        async {
            sqlx::query(CREATE_FILE_QUERY)
                .bind(self.id)
                .bind(self.user_id)
                .bind(self.name)
                .bind(self.size_bytes)
                .bind(self.parent_id)
                .bind(self.is_folder)
                .execute(&mut **tx)
                .await
                .map(|_| ())
        }
    }
}
