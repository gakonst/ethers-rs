use rustc_hex::{FromHex, ToHex};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Wrapper type around Vec<u8> to deserialize/serialize "0x" prefixed ethereum hex strings
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bytes(
    #[serde(
        serialize_with = "serialize_bytes",
        deserialize_with = "deserialize_bytes"
    )]
    pub Vec<u8>,
);

impl Bytes {
    /// Returns an empty bytes vector
    pub fn new() -> Self {
        Bytes(vec![])
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(src: Vec<u8>) -> Self {
        Self(src)
    }
}

pub fn serialize_bytes<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: AsRef<[u8]>,
{
    s.serialize_str(&format!("0x{}", x.as_ref().to_hex::<String>()))
}

pub fn deserialize_bytes<'de, D>(d: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(d)?;
    if value.len() >= 2 && &value[0..2] == "0x" {
        let bytes = FromHex::from_hex(&value[2..])
            .map_err(|e| Error::custom(format!("Invalid hex: {}", e)))?;
        Ok(bytes)
    } else {
        Err(Error::invalid_value(Unexpected::Str(&value), &"0x prefix"))
    }
}
