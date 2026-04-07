use serde::{Deserialize, Serialize, de::Visitor, ser::Error};

pub struct FixedSizedStr<const BYTES: usize> {
    len: usize,
    bytes: [u8; BYTES],
}

struct FixedStrVisitor<const BYTES: usize>;

#[derive(Debug)]
pub(crate) struct TooLong;

impl<const BYTES: usize> FixedSizedStr<BYTES> {
    pub(crate) fn new_from_str(value: &str) -> Result<Self, TooLong> {
        if value.len() > BYTES {
            return Err(TooLong);
        }

        let mut output: Self = Self {
            len: value.len(),
            bytes: [0; BYTES],
        };

        output.bytes[..value.len()].copy_from_slice(value.as_bytes());

        Ok(output)
    }
    pub fn as_str(&self) -> &str {
        // SAFETY: invariant maintained by constructor/deserializer
        unsafe { std::str::from_utf8_unchecked(&self.bytes[..self.len]) }
    }
}

impl<const BYTES: usize> Serialize for FixedSizedStr<BYTES> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(
            &str::from_utf8(&self.bytes[..self.len])
                .or_else(|err| Err(S::Error::custom(err.to_string())))?,
        )
    }
}

impl<'de, const BYTES: usize> Deserialize<'de> for FixedSizedStr<BYTES> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(FixedStrVisitor)
    }
}

impl<'de, const BYTES: usize> Visitor<'de> for FixedStrVisitor<BYTES> {
    type Value = FixedSizedStr<BYTES>;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Expected a String")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() > BYTES {
            return Err(E::invalid_length(v.len(), &"String is too long"));
        }

        let mut output: Self::Value = Self::Value {
            len: v.len(),
            bytes: [0; BYTES],
        };

        output.bytes[..v.len()].copy_from_slice(v.as_bytes());

        Ok(output)
    }
}
