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
SELECT id, name, email, is_admin, created_at, modified_at FROM users WHERE id = ? LIMIT 1;
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
                created_at: row.get(4),
                modified_at: row.get(5),
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

    use super::GetUserInfo;

    async fn setup(pool: &Pool<MySql>, email: &str) -> u32 {
        let name = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test").unwrap();
        let email_str = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str(email).unwrap();
        let user = UserCreationView::new(&name, &email_str);
        let hashed_pw = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test_password").unwrap();
        let mut tx = pool.begin().await.unwrap();
        let create = UserRepository::create(&user, &hashed_pw);
        create.execute(&mut tx).await.unwrap();
        create.commit(tx).await.unwrap();
        UserRepository::get_login_view(email)
            .read(pool)
            .await
            .unwrap()
            .unwrap()
            .id as u32
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        cleanup_user_by_email(pool, email).await;
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
