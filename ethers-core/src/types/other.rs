//! Support for capturing other fields
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::BTreeMap, ops::Deref};

/// A type that is supposed to capture additional fields that are not native to ethereum but included in ethereum adjacent networks, for example fields the [optimism `eth_getTransactionByHash` request](https://docs.alchemy.com/alchemy/apis/optimism/eth-gettransactionbyhash) returns additional fields that this type will capture
///
/// This type is supposed to be used with [`#[serde(flatten)`](https://serde.rs/field-attrs.html#flatten)
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OtherFields {
    /// Contains all unknown fields
    inner: BTreeMap<String, serde_json::Value>,
}

// === impl OtherFields ===

impl OtherFields {
    /// Returns the deserialized value of the field, if it exists.
    /// Deserializes the value with the given closure
    ///
    /// ```
    /// # use ethers_core::types::{OtherFields, U64};
    /// fn d(other: OtherFields) {
    ///  let l1_block_number = other.get_with("l1BlockNumber", |value| serde_json::from_value::<U64>(value)).unwrap().unwrap();
    /// # }
    /// ```
    pub fn get_with<F, V>(&self, key: impl AsRef<str>, with: F) -> Option<V>
    where
        V: DeserializeOwned,
        F: FnOnce(serde_json::Value) -> V,
    {
        self.inner.get(key.as_ref()).cloned().map(with)
    }

    /// Returns the deserialized value of the field, if it exists
    ///
    /// ```
    /// # use ethers_core::types::{OtherFields, U64};
    /// fn d(other: OtherFields) {
    ///  let l1_block_number = other.get_deserialized::<U64>("l1BlockNumber").unwrap().unwrap();
    /// # }
    /// ```
    pub fn get_deserialized<V: DeserializeOwned>(
        &self,
        key: impl AsRef<str>,
    ) -> Option<serde_json::Result<V>> {
        self.inner.get(key.as_ref()).cloned().map(serde_json::from_value)
    }

    /// Removes the deserialized value of the field, if it exists
    ///
    /// ```
    /// # use ethers_core::types::{OtherFields, U64};
    /// fn d(mut other: OtherFields) {
    ///  let l1_block_number = other.remove_deserialized::<U64>("l1BlockNumber").unwrap().unwrap();
    /// assert!(!other.contains_key("l1BlockNumber"));
    /// # }
    /// ```
    ///
    /// **Note:** this will also remove the value if deserializing it resulted in an error
    pub fn remove_deserialized<V: DeserializeOwned>(
        &mut self,
        key: impl AsRef<str>,
    ) -> Option<serde_json::Result<V>> {
        self.inner.remove(key.as_ref()).map(serde_json::from_value)
    }

    /// Removes the deserialized value of the field, if it exists.
    /// Deserializes the value with the given closure
    ///
    /// ```
    /// # use ethers_core::types::{OtherFields, U64};
    /// fn d(mut other: OtherFields) {
    ///  let l1_block_number = other.remove_with("l1BlockNumber", |value| serde_json::from_value::<U64>(value)).unwrap().unwrap();
    /// # }
    /// ```
    /// **Note:** this will also remove the value if deserializing it resulted in an error
    pub fn remove_with<F, V>(&mut self, key: impl AsRef<str>, with: F) -> Option<V>
    where
        V: DeserializeOwned,
        F: FnOnce(serde_json::Value) -> V,
    {
        self.inner.remove(key.as_ref()).map(with)
    }

    /// Removes the deserialized value of the field, if it exists and also returns the key
    ///
    /// ```
    /// # use ethers_core::types::{OtherFields, U64};
    /// fn d(mut other: OtherFields) {
    ///  let (key, l1_block_number_result) = other.remove_entry_deserialized::<U64>("l1BlockNumber").unwrap();
    /// let l1_block_number = l1_block_number_result.unwrap();
    /// assert!(!other.contains_key("l1BlockNumber"));
    /// # }
    /// ```
    ///
    /// **Note:** this will also remove the value if deserializing it resulted in an error
    pub fn remove_entry_deserialized<V: DeserializeOwned>(
        &mut self,
        key: impl AsRef<str>,
    ) -> Option<(String, serde_json::Result<V>)> {
        self.inner
            .remove_entry(key.as_ref())
            .map(|(key, value)| (key, serde_json::from_value(value)))
    }
}

impl Deref for OtherFields {
    type Target = BTreeMap<String, serde_json::Value>;

    #[inline]
    fn deref(&self) -> &BTreeMap<String, serde_json::Value> {
        self.as_ref()
    }
}

impl AsRef<BTreeMap<String, serde_json::Value>> for OtherFields {
    fn as_ref(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.inner
    }
}
