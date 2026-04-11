mod create;
mod delete;
mod email_exists;
mod get_login_view;
mod get_user_info;

use crate::data_definitions::{
    FixedSizedStr, MAX_UTF8_BYTES, StandardUserView, UserCreationView, UserLoginView,
};
use crate::database::{ReadOnly, Transactional};
use create::CreateUser;
use delete::DeleteUser;
use email_exists::EmailExists;
use get_login_view::GetLoginView;
use get_user_info::GetUserInfo;

pub(crate) struct UserRepository;

impl UserRepository {
    pub fn create<'a>(
        user: &'a UserCreationView,
        hashed_pw: &'a FixedSizedStr<MAX_UTF8_BYTES>,
    ) -> impl Transactional<Success = (), Error = sqlx::Error> {
        CreateUser::new(user, hashed_pw)
    }

    pub fn delete(user_id: i32) -> impl Transactional<Success = (), Error = sqlx::Error> {
        DeleteUser::new(user_id)
    }

    pub fn email_exists(email: &str) -> impl ReadOnly<Success = bool, Error = sqlx::Error> + '_ {
        EmailExists::new(email)
    }

    pub fn get_login_view(
        email: &str,
    ) -> impl ReadOnly<Success = Option<UserLoginView>, Error = sqlx::Error> + '_ {
        GetLoginView::new(email)
    }

    pub fn get_user_info(
        user_id: i32,
    ) -> impl ReadOnly<Success = Option<StandardUserView>, Error = sqlx::Error> {
        GetUserInfo::new(user_id)
    }
}
