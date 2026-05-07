use std::num::NonZero;

use crate::{
    ObjectID,
    data_definitions::{
        file::{EntryDraft, File},
        id::ID,
    },
    database::Transactional,
};
use sqlx::{error::Error, mysql::MySqlQueryResult};

const CREATE_FILE_QUERY: &str = r#"
INSERT INTO files (user_id, name, size_bytes, parent_id, status) VALUES (?, ?, ?, ?, ?);
"#;

pub(crate) struct CreateFile<'a>(&'a EntryDraft<'a, File>);

impl<'a> CreateFile<'a> {
    pub fn new(file: &'a EntryDraft<'a, File>) -> Self {
        Self(file)
    }
}

impl<'a> Transactional for CreateFile<'a> {
    type Success = ObjectID;
    type Error = Error;

    fn execute<'t>(
        &self,
        tx: &'t mut sqlx::Transaction<'_, sqlx::MySql>,
    ) -> impl Future<Output = Result<Self::Success, Self::Error>> + Send {
        async {
            let row: MySqlQueryResult = sqlx::query(CREATE_FILE_QUERY)
                .bind(self.0.get_user())
                .bind(self.0.get_name())
                .bind(self.0.get_size_bytes())
                .bind(self.0.get_parent().map(|id| id.0))
                .bind(self.0.get_state())
                .execute(&mut **tx)
                .await?;

            let id: NonZero<u32> = u32::try_from(row.last_insert_id())
                .ok()
                .and_then(NonZero::new)
                .ok_or_else(|| sqlx::Error::Protocol("Invalid insert ID".to_string()))?;

            Ok(ObjectID(ID(id)))
        }
    }
}
