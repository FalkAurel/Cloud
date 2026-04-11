use sqlx::{MySql, Pool, Row};

use crate::data_definitions::{FixedSizedStr, StandardUserView};
use crate::database::ReadOnly;

pub(super) struct GetUserInfo(i32);

impl GetUserInfo {
    pub const fn new(user_id: i32) -> Self {
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
        let row = sqlx::query(USER_INFO)
            .bind(self.0)
            .fetch_optional(pool)
            .await?;

        let result = match row {
            None => None,
            Some(row) => Some(StandardUserView {
                id: row.get("id"),
                name: FixedSizedStr::new_from_str(row.get::<&str, _>("name"))
                    .map_err(|e| sqlx::Error::ColumnDecode {
                        index: "name".to_string(),
                        source: Box::new(e),
                    })?,
                email: FixedSizedStr::new_from_str(row.get::<&str, _>("email"))
                    .map_err(|e| sqlx::Error::ColumnDecode {
                        index: "email".to_string(),
                        source: Box::new(e),
                    })?,
                is_admin: row.get("is_admin"),
                created_at: row.get("created_at"),
                modified_at: row.get("modified_at"),
            }),
        };

        Ok(result)
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

    async fn setup(pool: &Pool<MySql>, email: &str) -> i32 {
        let name = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test").unwrap();
        let email_str = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str(email).unwrap();
        let user = UserCreationView::new(&name, &email_str);
        let hashed_pw = FixedSizedStr::<MAX_UTF8_BYTES>::new_from_str("test_password").unwrap();
        let mut tx = pool.begin().await.unwrap();
        let create = UserRepository::create(&user, &hashed_pw);
        create.execute(&mut tx).await.unwrap();
        tx.commit().await.unwrap();
        UserRepository::get_login_view(email)
            .read(pool)
            .await
            .unwrap()
            .unwrap()
            .id
    }

    async fn cleanup(pool: &Pool<MySql>, email: &str) {
        cleanup_user_by_email(pool, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_some_for_existing_user() {
        let pool = init_db().await;
        let email = "userinfo@test.com";
        let id = setup(&pool, email).await;
        let result = GetUserInfo::new(id).read(&pool).await.unwrap();
        assert!(result.is_some());
        let view = result.unwrap();
        assert_eq!(view.email.as_str(), email);
        assert_eq!(view.name.as_str(), "test");
        assert!(!view.is_admin);
        cleanup(&pool, email).await;
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn returns_none_for_nonexistent_id() {
        let pool: Pool<MySql> = init_db().await;
        assert!(
            GetUserInfo::new(i32::MAX)
                .read(&pool)
                .await
                .unwrap()
                .is_none()
        );
    }
}
