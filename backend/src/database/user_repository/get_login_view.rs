use sqlx::{MySql, Pool, Row};

use crate::data_definitions::UserLoginView;
use crate::database::ReadOnly;

pub(super) struct GetLoginView<'a>(&'a str);

impl<'a> GetLoginView<'a> {
    pub const fn new(email: &'a str) -> Self {
        Self(email)
    }
}

const LOGIN_VIEW: &str = r#"
SELECT password AS password_hash, id FROM users WHERE email = ? LIMIT 1;
"#;

impl ReadOnly for GetLoginView<'_> {
    type Success = Option<UserLoginView>;
    type Error = sqlx::Error;

    async fn read(&self, pool: &Pool<MySql>) -> Result<Self::Success, Self::Error> {
        Ok(sqlx::query(LOGIN_VIEW)
            .bind(self.0)
            .fetch_optional(pool)
            .await?
            .map(|row| UserLoginView {
                id: row.get("id"),
                password_hash: row.get("password_hash"),
            }))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{MySql, Pool};

    use crate::{
        data_definitions::{FixedSizedStr, MAX_UTF8_BYTES, UserCreationView},
        database::{ReadOnly, Transactional, user_repository::UserRepository},
        init_db,
        test_harness_setup::cleanup_user_by_email,
    };

    use super::GetLoginView;

    async fn setup(pool: &Pool<MySql>, email: &str) {
        let name: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test").unwrap();
        let email_str: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str(email).unwrap();
        let user: UserCreationView<'_> = UserCreationView::new(&name, &email_str);
        let hashed_pw: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test_password").unwrap();
        let mut tx: sqlx::Transaction<'_, MySql> = pool.begin().await.unwrap();
        let create = UserRepository::create(&user, &hashed_pw);
        create.execute(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        cleanup_user_by_email(pool, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_some_for_existing_email() {
        let pool = init_db().await;
        let email = "loginview@test.com";
        setup(&pool, email).await;
        let result = GetLoginView::new(email).read(&pool).await.unwrap();
        assert!(result.is_some());
        let view = result.unwrap();
        assert!(view.id > 0);
        assert!(!view.password_hash.is_empty());
        cleanup(&pool, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_none_for_nonexistent_email() {
        let pool: Pool<MySql> = init_db().await;
        assert!(
            GetLoginView::new("ghost@test.com")
                .read(&pool)
                .await
                .unwrap()
                .is_none()
        );
    }
}
