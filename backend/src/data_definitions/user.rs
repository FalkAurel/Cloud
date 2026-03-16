use serde::{
    self, Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, Visitor},
    ser::SerializeStruct,
};

const DB_STRING_LENGTH: usize = 40;
const DB_STRING_BYTE_LENGTH: usize = DB_STRING_LENGTH * size_of::<char>();

#[derive(Debug)]
pub struct StandardUserView {
    name: [u8; DB_STRING_BYTE_LENGTH],
    email: [u8; DB_STRING_BYTE_LENGTH],
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
        Self::get_str(&self.name)
    }

    pub fn get_email(&self) -> &str {
        Self::get_str(&self.email)
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    #[inline]
    fn get_str(array: &[u8]) -> &str {
        let mut index: usize = 0;

        while index < array.len() && array[index] != b'\0' {
            index += 1;
        }

        unsafe { str::from_utf8_unchecked(&array[..index]) }
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
        let mut name: Option<[u8; DB_STRING_BYTE_LENGTH]> = None;
        let mut email: Option<[u8; DB_STRING_BYTE_LENGTH]> = None;
        let mut is_admin: Option<bool> = None;

        while let Some(key) = map.next_key::<StandardUserViewFields>()? {
            match key {
                StandardUserViewFields::Name if name.is_none() => {
                    let local_name: &str = map.next_value::<&str>()?;

                    let mut temp_name: [u8; DB_STRING_BYTE_LENGTH] = [b'\0'; DB_STRING_BYTE_LENGTH];
                    let len: usize = local_name.len();

                    if len > DB_STRING_BYTE_LENGTH {
                        return Err(A::Error::invalid_length(len, &"Name is too long"));
                    }

                    temp_name[..len.min(DB_STRING_BYTE_LENGTH)]
                        .copy_from_slice(local_name.as_bytes());

                    name = Some(temp_name);
                }
                StandardUserViewFields::Email if email.is_none() => {
                    let local_email: &str = map.next_value::<&str>()?;

                    let mut temp_email: [u8; DB_STRING_BYTE_LENGTH] =
                        [b'\0'; DB_STRING_BYTE_LENGTH];
                    let len: usize = local_email.len();

                    if len > DB_STRING_BYTE_LENGTH {
                        return Err(A::Error::invalid_length(len, &"Email is too long"));
                    }

                    temp_email[..len.min(DB_STRING_BYTE_LENGTH)]
                        .copy_from_slice(local_email.as_bytes());

                    email = Some(temp_email);
                }
                StandardUserViewFields::IsAdmin if is_admin.is_none() => {
                    is_admin = Some(map.next_value::<bool>()?)
                }
                _ => return Err(A::Error::duplicate_field(key.to_str())),
            }
        }

        let name: [u8; DB_STRING_BYTE_LENGTH] =
            name.ok_or_else(|| A::Error::missing_field("name"))?;
        let email: [u8; DB_STRING_BYTE_LENGTH] =
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

    use crate::data_definitions::user::{DB_STRING_BYTE_LENGTH, StandardUserView};
    const TEST_NAME: &'static str = "test";
    const TEST_EMAIL: &'static str = "test@gmail.com";

    #[test]
    fn serialize_user() {
        let mut name: [u8; DB_STRING_BYTE_LENGTH] = [b'\0'; DB_STRING_BYTE_LENGTH];
        let mut email: [u8; DB_STRING_BYTE_LENGTH] = [b'\0'; DB_STRING_BYTE_LENGTH];

        for (index, c) in TEST_NAME.bytes().enumerate() {
            name[index] = c;
        }

        for (index, c) in TEST_EMAIL.bytes().enumerate() {
            email[index] = c;
        }

        let user: StandardUserView = StandardUserView {
            name,
            email,
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
        let long_name: String = "𐍈".repeat(40);

        assert_eq!(long_name.chars().count(), 40);
        assert_eq!(long_name.as_bytes().len(), DB_STRING_BYTE_LENGTH);

        let mut name: [u8; DB_STRING_BYTE_LENGTH] = [b'\0'; DB_STRING_BYTE_LENGTH];
        let mut email: [u8; DB_STRING_BYTE_LENGTH] = [b'\0'; DB_STRING_BYTE_LENGTH];

        for (index, c) in long_name.bytes().enumerate() {
            name[index] = c;
        }

        for (index, c) in TEST_EMAIL.bytes().enumerate() {
            email[index] = c;
        }

        let user: StandardUserView = StandardUserView {
            name,
            email,
            is_admin: false,
        };

        let recovered_user: StandardUserView =
            json::from_str(&json::to_string(&user).unwrap()).unwrap();
        assert_eq!(recovered_user.get_name(), long_name);
    }
}
