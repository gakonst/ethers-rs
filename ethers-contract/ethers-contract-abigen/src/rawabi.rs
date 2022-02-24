//! This is a basic representation of a contract ABI that does no post processing but contains the
//! raw content of the ABI.

#![allow(missing_docs)]
use serde::{
    de::{MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

/// Contract ABI as a list of items where each item can be a function, constructor or event
#[derive(Debug, Clone, Serialize)]
#[serde(transparent)]
pub struct RawAbi(Vec<Item>);

impl IntoIterator for RawAbi {
    type Item = Item;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

struct RawAbiVisitor;

impl<'de> Visitor<'de> for RawAbiVisitor {
    type Value = RawAbi;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence or map with `abi` key")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut vec = Vec::new();

        while let Some(element) = seq.next_element()? {
            vec.push(element);
        }

        Ok(RawAbi(vec))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut vec = None;

        while let Some(key) = map.next_key::<String>()? {
            if key == "abi" {
                vec = Some(RawAbi(map.next_value::<Vec<Item>>()?));
            } else {
                map.next_value::<serde::de::IgnoredAny>()?;
            }
        }

        vec.ok_or_else(|| serde::de::Error::missing_field("abi"))
    }
}

impl<'de> Deserialize<'de> for RawAbi {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(RawAbiVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    #[serde(default)]
    pub inputs: Vec<Component>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_mutability: Option<String>,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default)]
    pub outputs: Vec<Component>,
    // required to satisfy solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anonymous: Option<bool>,
}

/// Either an input/output or a nested component of an input/output
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Component {
    #[serde(rename = "internalType", default, skip_serializing_if = "Option::is_none")]
    pub internal_type: Option<String>,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub type_field: String,
    #[serde(default)]
    pub components: Vec<Component>,
    /// Indexed flag. for solidity events
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub indexed: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::abi::Abi;

    #[test]
    fn can_parse_raw_abi() {
        const VERIFIER_ABI: &str = include_str!("../../tests/solidity-contracts/verifier_abi.json");
        let _ = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();
    }

    #[test]
    fn can_parse_hardhat_raw_abi() {
        const VERIFIER_ABI: &str =
            include_str!("../../tests/solidity-contracts/verifier_abi_hardhat.json");
        let _ = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();
    }

    /// due to ethabi's limitations some may be stripped when ethers-solc generates the abi, such as
    /// the name of the component
    #[test]
    fn can_parse_ethers_solc_generated_abi() {
        let s = r#"[{"type":"function","name":"greet","inputs":[{"internalType":"struct Greeter.Stuff","name":"stuff","type":"tuple","components":[{"type":"bool"}]}],"outputs":[{"internalType":"struct Greeter.Stuff","name":"","type":"tuple","components":[{"type":"bool"}]}],"stateMutability":"view"}]"#;
        let _ = serde_json::from_str::<RawAbi>(s).unwrap();
    }

    #[test]
    fn can_ethabi_round_trip() {
        let s = r#"[{"anonymous":false,"inputs":[{"indexed":true,"internalType":"uint64","name":"number","type":"uint64"}],"name":"MyEvent","type":"event"},{"inputs":[],"name":"greet","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;

        let raw = serde_json::from_str::<RawAbi>(s).unwrap();
        let abi = serde_json::from_str::<Abi>(s).unwrap();
        let de = serde_json::to_string(&raw).unwrap();
        assert_eq!(abi, serde_json::from_str::<Abi>(&de).unwrap());
    }
}
