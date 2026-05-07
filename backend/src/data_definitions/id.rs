use std::{
    fmt,
    num::{NonZero, ParseIntError},
};

use rocket::request::FromParam;
use serde::{Deserialize, Serialize};
use sqlx::{MySql, ValueRef};
#[cfg(feature = "export_binding")]
use ts_rs::TS;

#[cfg_attr(feature = "export_binding", derive(TS))]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Copy)]
#[repr(transparent)]
pub struct ID(pub(crate) NonZero<u32>);

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromParam<'_> for ID {
    type Error = ParseIntError;
    fn from_param(param: &'_ str) -> Result<Self, Self::Error> {
        Ok(ID(param.parse::<NonZero<u32>>()?))
    }
}

impl sqlx::Decode<'_, MySql> for ID {
    fn decode(
        value: <MySql as sqlx::Database>::ValueRef<'_>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        if value.is_null() {
            return Err("ID cannot be null".into());
        }
        let n: u32 = u32::decode(value)?;
        NonZero::new(n)
            .map(ID)
            .ok_or_else(|| "ID must be non-zero".into())
    }
}

impl sqlx::Encode<'_, MySql> for ID {
    fn encode(
        self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError>
    where
        Self: Sized,
    {
        NonZero::<u32>::encode(self.0, buf)
    }

    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'_>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        NonZero::<u32>::encode_by_ref(&self.0, buf)
    }
}

impl sqlx::Type<MySql> for ID {
    fn type_info() -> <MySql as sqlx::Database>::TypeInfo {
        u32::type_info()
    }
}
