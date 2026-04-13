use sqlx::{Error, MySql, Transaction};

use crate::database::Transactional;

pub(crate) struct DeleteUser(i32);

impl DeleteUser {
    pub const fn new(user_id: i32) -> Self {
        Self(user_id)
    }
}

const DELETE_USER: &str = r#"
DELETE FROM users WHERE id = ?;
"#;

impl Transactional for DeleteUser {
    type Success = ();
    type Error = sqlx::Error;

    async fn execute<'t>(
        &self,
        tx: &'t mut Transaction<'_, MySql>,
    ) -> Result<Self::Success, Self::Error> {
        let rows_affected = sqlx::query(DELETE_USER)
            .bind(self.0)
            .execute(&mut **tx)
            .await?
            .rows_affected();

        if rows_affected == 0 {
            return Err(Error::RowNotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data_definitions::{FixedSizedStr, MAX_UTF8_BYTES, UserCreationView},
        database::{
            ReadOnly, Transactional,
            user_repository::{UserRepository, create::CreateUser, delete::DeleteUser},
        },
        init_db,
        test_harness_setup::cleanup_user_by_email,
    };

    async fn get_id(pool: &sqlx::Pool<sqlx::MySql>, email: &str) -> i32 {
        UserRepository::get_login_view(email)
            .read(pool)
            .await
            .unwrap()
            .unwrap()
            .id
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn delete_user() {
        let email: &str = "delete@test.com";
        let name: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str("test").unwrap();
        let email_fixed_str: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str(email).unwrap();

        let user: UserCreationView = UserCreationView::new(&name, &email_fixed_str);
        let hashed_pw: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str("test_password").unwrap();

        let pool: sqlx::Pool<sqlx::MySql> = init_db().await;

        let mut tx = pool.begin().await.unwrap();
        let create_user = CreateUser::new(&user, &hashed_pw);
        create_user.execute(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        let id = get_id(&pool, email).await;

        let mut tx = pool.begin().await.unwrap();
        let delete_user = DeleteUser::new(id);
        assert!(delete_user.execute(&mut tx).await.is_ok());
        assert!(tx.commit().await.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn delete_non_existent_user() {
        let pool = init_db().await;
        let mut tx = pool.begin().await.unwrap();
        let delete_user = DeleteUser::new(i32::MAX);
        assert!(delete_user.execute(&mut tx).await.is_err());
        assert!(tx.rollback().await.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires database"]
    async fn rollback_preserves_user() {
        let email: &str = "delete_rollback@test.com";
        let name: FixedSizedStr<MAX_UTF8_BYTES> = FixedSizedStr::new_from_str("test").unwrap();
        let hashed_email: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str(email).unwrap();

        let user: UserCreationView = UserCreationView::new(&name, &hashed_email);
        let hashed_pw: FixedSizedStr<MAX_UTF8_BYTES> =
            FixedSizedStr::new_from_str("test_password").unwrap();

        let pool: sqlx::Pool<sqlx::MySql> = init_db().await;

        let mut tx = pool.begin().await.unwrap();
        let create = CreateUser::new(&user, &hashed_pw);
        create.execute(&mut tx).await.unwrap();
        tx.commit().await.unwrap();

        let id = get_id(&pool, email).await;

        let mut tx = pool.begin().await.unwrap();
        let delete_user = DeleteUser::new(id);
        delete_user.execute(&mut tx).await.unwrap();
        tx.rollback().await.unwrap();

        assert!(
            UserRepository::email_exists(email)
                .read(&pool)
                .await
                .unwrap()
        );

        cleanup_user_by_email(&pool, email).await;
    }
}
