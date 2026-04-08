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

    use crate::{database::ReadOnly, init_db};

    use super::GetLoginView;

    async fn setup(pool: &Pool<MySql>, email: &str) {
        sqlx::query("INSERT INTO users (name, email, password) VALUES (?, ?, ?)")
            .bind("test")
            .bind(email)
            .bind("test_password")
            .execute(pool)
            .await
            .unwrap();
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        sqlx::query("DELETE FROM users WHERE email = ?")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
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
    async fn returns_none_for_nonexistent_email() {
        let pool = init_db().await;
        assert!(
            GetLoginView::new("ghost@test.com")
                .read(&pool)
                .await
                .unwrap()
                .is_none()
        );
    }
}
