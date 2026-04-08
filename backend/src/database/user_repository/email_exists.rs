use sqlx::{MySql, Pool, Row};

use crate::database::ReadOnly;

pub(super) struct EmailExists<'a>(&'a str);

impl<'a> EmailExists<'a> {
    pub const fn new(email: &'a str) -> Self {
        Self(email)
    }
}

const EMAIL_EXISTS: &str = r#"
SELECT EXISTS(SELECT 1 FROM users WHERE email = ?);
"#;

impl ReadOnly for EmailExists<'_> {
    type Success = bool;
    type Error = sqlx::Error;

    async fn read(&self, pool: &Pool<MySql>) -> Result<Self::Success, Self::Error> {
        Ok(sqlx::query(EMAIL_EXISTS)
            .bind(self.0)
            .fetch_one(pool)
            .await?
            .get::<bool, usize>(0))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{MySql, Pool};

    use crate::{database::ReadOnly, init_db};

    use super::EmailExists;

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
    async fn returns_true_for_existing_email() {
        let pool = init_db().await;
        let email = "exists@test.com";
        setup(&pool, email).await;
        assert!(EmailExists::new(email).read(&pool).await.unwrap());
        cleanup(&pool, email).await;
    }

    #[tokio::test]
    async fn returns_false_for_nonexistent_email() {
        let pool = init_db().await;
        assert!(
            !EmailExists::new("ghost@test.com")
                .read(&pool)
                .await
                .unwrap()
        );
    }
}
