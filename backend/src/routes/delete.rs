use crate::{
    data_definitions::{Auth, JWT, StandardUserView},
    database::{ReadOnly, Transactional, user_repository::UserRepository},
};
use rocket::{State, delete, http::Status};
use sqlx::{MySql, Pool, Transaction};

#[delete("/delete/<id>")]
pub async fn delete(
    id: u32,
    auth: Auth,
    db: &State<Pool<MySql>>,
) -> Result<Status, (Status, &'static str)> {
    let jwt: JWT = auth.get_jwt();

    if jwt.user_id == id {
        return delete_user(id, db).await;
    }

    match UserRepository::get_user_info(jwt.user_id).read(db).await {
        Ok(Some(StandardUserView { is_admin: true, .. })) => {
            delete_user(id, db).await
        }
        Ok(Some(StandardUserView {
            is_admin: false, ..
        })) => {
            todo!("Implement Logging and return a proper Error message")
        }
        Ok(None) => {
            todo!("Implement Logging and return a proper Error message")
        }
        Err(err) => {
            todo!("Implement Logging and return a proper Error message")
        }
    }
}

async fn delete_user(id: u32, db: &Pool<MySql>) -> Result<Status, (Status, &'static str)> {
        let mut transaction: Transaction<MySql> = db.begin().await.expect("Implement proper error handling");
        let delete_user = UserRepository::delete(id);
        delete_user.execute(&mut transaction).await.expect("Implement proper error handling");
        delete_user.commit(transaction).await.expect("Implement proper error handling");

        Ok(Status::Ok)
}
