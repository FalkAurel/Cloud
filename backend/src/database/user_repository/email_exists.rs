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

    use crate::{
        data_definitions::{FixedSizedStr, MAX_UTF8_BYTES, UserCreationView},
        database::{ReadOnly, Transactional, user_repository::UserRepository},
        init_db,
        test_harness_setup::cleanup_user_by_email,
    };

    use super::EmailExists;

    async fn setup(pool: &Pool<MySql>, email: &str) {
        let name = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test").unwrap();
        let email_str = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str(email).unwrap();
        let user = UserCreationView::new(&name, &email_str);
        let hashed_pw = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test_password").unwrap();
        let mut tx = pool.begin().await.unwrap();
        let create = UserRepository::create(&user, &hashed_pw);
        create.execute(&mut tx).await.unwrap();
        create.commit(tx).await.unwrap();
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        cleanup_user_by_email(pool, email).await;
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
