use sqlx::{MySql, Pool, Row};

use crate::data_definitions::{FixedSizedStr, StandardUserView};
use crate::database::ReadOnly;

pub(super) struct GetUserInfo(u32);

impl GetUserInfo {
    pub const fn new(user_id: u32) -> Self {
        Self(user_id)
    }
}

const USER_INFO: &str = r#"
SELECT id, name, email, is_admin FROM users WHERE id = ? LIMIT 1;
"#;

impl ReadOnly for GetUserInfo {
    type Success = Option<StandardUserView>;
    type Error = sqlx::Error;

    async fn read(&self, pool: &Pool<MySql>) -> Result<Self::Success, Self::Error> {
        Ok(sqlx::query(USER_INFO)
            .bind(self.0 as i32)
            .fetch_optional(pool)
            .await?
            .map(|row| StandardUserView {
                id: row.get::<i32, usize>(0),
                name: FixedSizedStr::new_from_str(row.get::<&str, usize>(1))
                    .expect("DB contains invalid name that passed signup validation"),
                email: FixedSizedStr::new_from_str(row.get::<&str, usize>(2))
                    .expect("DB contains invalid email that passed signup validation"),
                is_admin: row.get(3),
            }))
    }
}

#[cfg(test)]
mod tests {
    use sqlx::{MySql, Pool, Row};

    use crate::{database::ReadOnly, init_db};

    use super::GetUserInfo;

    async fn setup(pool: &Pool<MySql>, email: &str) -> i32 {
        sqlx::query("INSERT INTO users (name, email, password) VALUES (?, ?, ?)")
            .bind("test")
            .bind(email)
            .bind("test_password")
            .execute(pool)
            .await
            .unwrap();

        sqlx::query("SELECT id FROM users WHERE email = ? LIMIT 1")
            .bind(email)
            .fetch_one(pool)
            .await
            .unwrap()
            .get::<i32, usize>(0)
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        sqlx::query("DELETE FROM users WHERE email = ?")
            .bind(email)
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn returns_some_for_existing_user() {
        let pool = init_db().await;
        let email = "userinfo@test.com";
        let id = setup(&pool, email).await;
        let result = GetUserInfo::new(id as u32).read(&pool).await.unwrap();
        assert!(result.is_some());
        let view = result.unwrap();
        assert_eq!(view.email.as_str(), email);
        assert_eq!(view.name.as_str(), "test");
        assert!(!view.is_admin);
        cleanup(&pool, email).await;
    }

    #[tokio::test]
    async fn returns_none_for_nonexistent_id() {
        let pool: Pool<MySql> = init_db().await;
        assert!(
            GetUserInfo::new(u32::MAX)
                .read(&pool)
                .await
                .unwrap()
                .is_none()
        );
    }
}
