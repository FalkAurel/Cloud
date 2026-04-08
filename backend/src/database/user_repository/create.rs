use sqlx::{MySql, Transaction};

use crate::data_definitions::{FixedSizedStr, MAX_UTF8_BYTES, UserCreationView};
use crate::database::Transactional;

pub(crate) struct CreateUser<'a>(&'a UserCreationView<'a>, &'a FixedSizedStr<MAX_UTF8_BYTES>);

impl<'a> CreateUser<'a> {
    pub const fn new(
        user: &'a UserCreationView,
        hashed_pw: &'a FixedSizedStr<MAX_UTF8_BYTES>,
    ) -> Self {
        Self(user, hashed_pw)
    }
}

const CREATE_USER: &str = r#"
INSERT INTO users (name, email, password) VALUES (?, ?, ?);
"#;

impl Transactional for CreateUser<'_> {
    type Success = ();
    type Error = sqlx::Error;

    async fn execute<'t>(
        &self,
        tx: &'t mut Transaction<'_, MySql>,
    ) -> Result<Self::Success, Self::Error> {
        sqlx::query(CREATE_USER)
            .bind(self.0.get_name())
            .bind(self.0.get_email())
            .bind(self.1.as_str())
            .execute(&mut **tx)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_definitions::{FixedSizedStr, MAX_UTF8_BYTES, UserCreationView},
        database::{
            ReadOnly, Transactional,
            user_repository::{UserRepository, create::CreateUser},
        },
        init_db,
    };

    #[tokio::test]
    async fn create_user() {
        let name: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str("test").unwrap();
        let email: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str("test").unwrap();
        let user: UserCreationView<'_> = UserCreationView::new(&name, &email);

        let hashed_pw: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str("test_password").unwrap();

        let pool = init_db().await;
        let mut tx = pool.begin().await.unwrap();
        let create_user = CreateUser::new(&user, &hashed_pw);
        create_user.execute(&mut tx).await.unwrap();
        assert!(CreateUser::rollback(tx).await.is_ok());
    }

    #[tokio::test]
    async fn rollback_does_not_persist_user() {
        let email = "create_rollback@test.com";
        let fix_sized_str: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str(email).unwrap();
        let name: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str("test").unwrap();
        let user: UserCreationView = UserCreationView::new(&name, &fix_sized_str);
        let hashed_pw: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str("test_password").unwrap();

        let pool = init_db().await;
        let mut tx = pool.begin().await.unwrap();
        CreateUser::new(&user, &hashed_pw)
            .execute(&mut tx)
            .await
            .unwrap();
        CreateUser::rollback(tx).await.unwrap();

        assert!(
            !UserRepository::email_exists(email)
                .read(&pool)
                .await
                .unwrap()
        );
    }
}
