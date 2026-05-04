use crate::{
    ObjectID,
    data_definitions::file::{Directory, EntryDraft},
    data_definitions::id::ID,
    database::Transactional,
};
use std::num::NonZero;

const CREATE_FOLDER_QUERY: &str = r#"
INSERT INTO files (user_id, name, size_bytes, parent_id, status) VALUES (?, ?, ?, ?, ?);
"#;

pub(crate) struct CreateFolder<'a>(&'a EntryDraft<'a, Directory>);

impl<'a> CreateFolder<'a> {
    pub(crate) fn new(folder: &'a EntryDraft<'a, Directory>) -> Self {
        Self(folder)
    }
}

impl<'a> Transactional for CreateFolder<'a> {
    type Success = ObjectID;
    type Error = sqlx::Error;
    fn execute<'t>(
        &self,
        tx: &'t mut sqlx::Transaction<'_, sqlx::MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send {
        async {
            let row: sqlx::mysql::MySqlQueryResult = sqlx::query(CREATE_FOLDER_QUERY)
                .bind(self.0.get_user())
                .bind(self.0.get_name())
                .bind(self.0.get_size_bytes())
                .bind(self.0.get_parent().map(|id| id.0))
                .execute(&mut **tx)
                .await?;

            let id: NonZero<u32> = u32::try_from(row.last_insert_id())
                .ok()
                .and_then(NonZero::new)
                .ok_or_else(|| sqlx::Error::Protocol("Invalid insert ID".into()))?;

            Ok(ObjectID(ID(id)))
        }
    }
}
