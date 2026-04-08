use serde::{self, Deserialize, Serialize};
#[cfg(feature = "export_binding")]
use ts_rs::TS;

use crate::data_definitions::FixedSizedStr;

const DB_STRING_LENGTH: usize = 40;
pub(crate) const MAX_UTF8_BYTES: usize = DB_STRING_LENGTH * size_of::<char>();

#[cfg_attr(feature = "export_binding", derive(TS))]
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct StandardUserView {
    pub(crate) id: i32,
    #[cfg_attr(feature = "export_binding", ts(type = "string"))]
    pub(crate) name: FixedSizedStr<MAX_UTF8_BYTES>,
    #[cfg_attr(feature = "export_binding", ts(type = "string"))]
    pub(crate) email: FixedSizedStr<MAX_UTF8_BYTES>,
    pub(crate) is_admin: bool,
}

impl StandardUserView {
    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_email(&self) -> &str {
        self.email.as_str()
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }
}

pub(crate) struct UserCreationView<'a> {
    name: &'a FixedSizedStr<MAX_UTF8_BYTES>,
    email: &'a FixedSizedStr<MAX_UTF8_BYTES>,
}

impl<'a> UserCreationView<'a> {
    pub fn new(
        name: &'a FixedSizedStr<MAX_UTF8_BYTES>,
        email: &'a FixedSizedStr<MAX_UTF8_BYTES>,
    ) -> Self {
        Self { name, email }
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_email(&self) -> &str {
        self.email.as_str()
    }
}

#[cfg_attr(feature = "export_binding", derive(TS))]
#[derive(Deserialize)]
pub struct UserLoginRequest<'a> {
    pub(crate) email: &'a str,
    pub(crate) password: &'a str,
}

pub(crate) struct UserLoginView {
    pub(crate) id: i32,
    pub(crate) password_hash: String,
}

#[cfg_attr(feature = "export_binding", derive(TS))]
#[derive(Deserialize)]
pub struct UserSignupRequest<'a> {
    pub(crate) email: &'a str,
    pub(crate) password: &'a str,
    pub(crate) name: &'a str,
}

#[cfg(test)]
mod user {
    use super::StandardUserView;
    use rocket::serde::json;

    use crate::data_definitions::{
        FixedSizedStr,
        user::{DB_STRING_LENGTH, MAX_UTF8_BYTES},
    };
    const TEST_NAME: &'static str = "test";
    const TEST_EMAIL: &'static str = "test@gmail.com";

    #[test]
    fn serialize_user() {
        let user: StandardUserView = StandardUserView {
            id: 0,
            name: FixedSizedStr::new_from_str(TEST_NAME).unwrap(),
            email: FixedSizedStr::new_from_str(TEST_EMAIL).unwrap(),
            is_admin: false,
        };

        let expected_json: &str =
            r#"{"id":0,"name":"test","email":"test@gmail.com","is_admin":false}"#;
        assert_eq!(expected_json, json::to_string(&user).unwrap())
    }

    #[test]
    fn deserialize_user() {
        let json_user: &str = r#"
        {
            "id": 0,
            "name": "test",
            "email": "test@gmail.com",
            "is_admin": false
        }
        "#;

        let user: StandardUserView = json::from_str::<StandardUserView>(json_user).unwrap();

        assert_eq!(user.get_name(), TEST_NAME);
        assert_eq!(user.get_email(), TEST_EMAIL);
        assert_eq!(user.is_admin, false);
    }

    #[test]
    fn deserialize_and_serialize_utf8() {
        let long_name: String = "𐍈".repeat(DB_STRING_LENGTH);

        assert_eq!(long_name.chars().count(), 40);
        assert_eq!(long_name.as_bytes().len(), MAX_UTF8_BYTES);

        let user: StandardUserView = StandardUserView {
            id: 0,
            name: FixedSizedStr::new_from_str(&long_name).unwrap(),
            email: FixedSizedStr::new_from_str(TEST_EMAIL).unwrap(),
            is_admin: false,
        };

        let recovered_user: StandardUserView =
            json::from_str(&json::to_string(&user).unwrap()).unwrap();
        assert_eq!(recovered_user.get_name(), long_name);
    }
}
