//! serde helpers

use ethers_core::types::Bytes;
use serde::{Deserialize, Deserializer};

pub fn deserialize_bytes<'de, D>(d: D) -> std::result::Result<Bytes, D::Error>
where
    D: Deserializer<'de>,
{
    String::deserialize(d)?.parse::<Bytes>().map_err(|e| serde::de::Error::custom(e.to_string()))
}

pub fn deserialize_opt_bytes<'de, D>(d: D) -> std::result::Result<Option<Bytes>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(d)?;
    if let Some(value) = value {
        Ok(Some(
            if let Some(value) = value.strip_prefix("0x") {
                hex::decode(value)
            } else {
                hex::decode(&value)
            }
            .map_err(|e| serde::de::Error::custom(e.to_string()))?
            .into(),
        ))
    } else {
        Ok(None)
    }
}

pub fn default_for_null<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + Default,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

pub mod json_string_opt {
    use serde::{
        de::{self, DeserializeOwned},
        ser, Deserialize, Deserializer, Serialize, Serializer,
    };

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        if let Some(value) = value {
            let value = serde_json::to_string(value).map_err(ser::Error::custom)?;
            serializer.serialize_str(&value)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: DeserializeOwned,
    {
        if let Some(s) = Option::<String>::deserialize(deserializer)? {
            serde_json::from_str(&s).map_err(de::Error::custom).map(Some)
        } else {
            Ok(None)
        }
    }
}

/// serde support for string
pub mod string_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.starts_with("0x") {
            serializer.serialize_str(value.as_str())
        } else {
            serializer.serialize_str(&format!("0x{}", value))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if let Some(rem) = value.strip_prefix("0x") {
            Ok(rem.to_string())
        } else {
            Ok(value)
        }
    }
}

pub mod display_from_str_opt {
    use serde::{de, Deserialize, Deserializer, Serializer};
    use std::{fmt, str::FromStr};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        if let Some(value) = value {
            serializer.collect_str(value)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
        T::Err: fmt::Display,
    {
        if let Some(s) = Option::<String>::deserialize(deserializer)? {
            s.parse().map_err(de::Error::custom).map(Some)
        } else {
            Ok(None)
        }
    }
}
