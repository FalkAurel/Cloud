use serde::{
    self, Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
    ser::SerializeStruct,
};

use crate::data_definitions::FixedSizedStr;

const DB_STRING_LENGTH: usize = 40;
const MAX_UTF8_BYTES: usize = DB_STRING_LENGTH * size_of::<char>();

pub struct StandardUserView {
    name: FixedSizedStr<MAX_UTF8_BYTES>,
    email: FixedSizedStr<MAX_UTF8_BYTES>,
    is_admin: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum StandardUserViewFields {
    Name,
    Email,
    IsAdmin,
}

struct StandardUserViewDeserializer;

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

impl StandardUserViewFields {
    pub const fn to_str(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Email => "email",
            Self::IsAdmin => "is_admin",
        }
    }
}

impl Serialize for StandardUserView {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state: <S as Serializer>::SerializeStruct =
            serializer.serialize_struct("StandardUserView", 3)?;
        state.serialize_field("name", self.get_name())?;
        state.serialize_field("email", self.get_email())?;
        state.serialize_field("is_admin", &self.is_admin)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for StandardUserView {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct(
            "StandardUserView",
            &["name", "email", "is_admin"],
            StandardUserViewDeserializer,
        )
    }
}

impl<'de> Visitor<'de> for StandardUserViewDeserializer {
    type Value = StandardUserView;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expecting struct `StandardUserView`")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut name: Option<FixedSizedStr<MAX_UTF8_BYTES>> = None;
        let mut email: Option<FixedSizedStr<MAX_UTF8_BYTES>> = None;
        let mut is_admin: Option<bool> = None;

        while let Some(key) = map.next_key::<StandardUserViewFields>()? {
            match key {
                StandardUserViewFields::Name if name.is_none() => {
                    name = Some(map.next_value::<FixedSizedStr<MAX_UTF8_BYTES>>()?);
                }
                StandardUserViewFields::Email if email.is_none() => {
                    email = Some(map.next_value::<FixedSizedStr<MAX_UTF8_BYTES>>()?)
                }
                StandardUserViewFields::IsAdmin if is_admin.is_none() => {
                    is_admin = Some(map.next_value::<bool>()?)
                }
                _ => return Err(A::Error::duplicate_field(key.to_str())),
            }
        }

        let name: FixedSizedStr<MAX_UTF8_BYTES> =
            name.ok_or_else(|| A::Error::missing_field("name"))?;
        let email: FixedSizedStr<MAX_UTF8_BYTES> =
            email.ok_or_else(|| A::Error::missing_field("email"))?;
        let is_admin: bool = is_admin.ok_or_else(|| A::Error::missing_field("is_admin"))?;

        Ok(StandardUserView {
            name,
            email,
            is_admin,
        })
    }
}

#[cfg(test)]
mod user {
    use rocket::serde::json;

    use crate::data_definitions::{
        FixedSizedStr,
        user::{DB_STRING_LENGTH, MAX_UTF8_BYTES, StandardUserView},
    };
    const TEST_NAME: &'static str = "test";
    const TEST_EMAIL: &'static str = "test@gmail.com";

    #[test]
    fn serialize_user() {
        let user: StandardUserView = StandardUserView {
            name: FixedSizedStr::new_from_str(TEST_NAME).unwrap(),
            email: FixedSizedStr::new_from_str(TEST_EMAIL).unwrap(),
            is_admin: false,
        };

        let expected_json: &str = r#"{"name":"test","email":"test@gmail.com","is_admin":false}"#;
        assert_eq!(expected_json, json::to_string(&user).unwrap())
    }

    #[test]
    fn deserialize_user() {
        let json_user: &str = r#"
        {
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
            name: FixedSizedStr::new_from_str(&long_name).unwrap(),
            email: FixedSizedStr::new_from_str(TEST_EMAIL).unwrap(),
            is_admin: false,
        };

        let recovered_user: StandardUserView =
            json::from_str(&json::to_string(&user).unwrap()).unwrap();
        assert_eq!(recovered_user.get_name(), long_name);
    }
}
