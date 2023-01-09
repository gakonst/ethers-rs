//! This is a basic representation of a contract ABI that does no post processing but contains the
//! raw content of the ABI.

#![allow(missing_docs)]
use crate::types::Bytes;
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

/// Represents contract ABI input variants
#[derive(Deserialize)]
#[serde(untagged)]
pub enum JsonAbi {
    /// json object input as `{"abi": [..], "bin": "..."}`
    Object(AbiObject),
    /// json array input as `[]`
    #[serde(deserialize_with = "deserialize_abi_array")]
    Array(RawAbi),
}

// === impl JsonAbi ===

impl JsonAbi {
    /// Returns the bytecode object
    pub fn bytecode(&self) -> Option<Bytes> {
        match self {
            JsonAbi::Object(abi) => abi.bytecode.clone(),
            JsonAbi::Array(_) => None,
        }
    }

    /// Returns the deployed bytecode object
    pub fn deployed_bytecode(&self) -> Option<Bytes> {
        match self {
            JsonAbi::Object(abi) => abi.deployed_bytecode.clone(),
            JsonAbi::Array(_) => None,
        }
    }
}

fn deserialize_abi_array<'de, D>(deserializer: D) -> Result<RawAbi, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(RawAbiVisitor)
}

/// Contract ABI and optional bytecode as JSON object
pub struct AbiObject {
    pub abi: RawAbi,
    pub bytecode: Option<Bytes>,
    pub deployed_bytecode: Option<Bytes>,
}

struct AbiObjectVisitor;

impl<'de> Visitor<'de> for AbiObjectVisitor {
    type Value = AbiObject;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a sequence or map with `abi` key")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut abi = None;
        let mut bytecode = None;
        let mut deployed_bytecode = None;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Bytecode {
            Object { object: Bytes },
            Bytes(Bytes),
        }

        impl Bytecode {
            fn into_bytes(self) -> Option<Bytes> {
                let bytecode = match self {
                    Bytecode::Object { object } => object,
                    Bytecode::Bytes(bytes) => bytes,
                };
                if bytecode.is_empty() {
                    None
                } else {
                    Some(bytecode)
                }
            }
        }

        /// represents nested bytecode objects of the `evm` value
        #[derive(Deserialize)]
        struct EvmObj {
            bytecode: Option<Bytecode>,
            #[serde(rename = "deployedBytecode")]
            deployed_bytecode: Option<Bytecode>,
        }

        struct DeserializeBytes(Bytes);

        impl<'de> Deserialize<'de> for DeserializeBytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(DeserializeBytes(crate::types::deserialize_bytes(deserializer)?.into()))
            }
        }

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "abi" => {
                    abi = Some(RawAbi(map.next_value::<Vec<Item>>()?));
                }
                "evm" => {
                    if let Ok(evm) = map.next_value::<EvmObj>() {
                        bytecode = evm.bytecode.and_then(|b| b.into_bytes());
                        deployed_bytecode = evm.deployed_bytecode.and_then(|b| b.into_bytes())
                    }
                }
                "bytecode" | "byteCode" => {
                    bytecode = map.next_value::<Bytecode>().ok().and_then(|b| b.into_bytes());
                }
                "deployedbytecode" | "deployedBytecode" => {
                    deployed_bytecode =
                        map.next_value::<Bytecode>().ok().and_then(|b| b.into_bytes());
                }
                "bin" => {
                    bytecode = map
                        .next_value::<DeserializeBytes>()
                        .ok()
                        .map(|b| b.0)
                        .filter(|b| !b.0.is_empty());
                }
                "runtimebin" | "runtimeBin" => {
                    deployed_bytecode = map
                        .next_value::<DeserializeBytes>()
                        .ok()
                        .map(|b| b.0)
                        .filter(|b| !b.0.is_empty());
                }
                _ => {
                    map.next_value::<serde::de::IgnoredAny>()?;
                }
            }
        }

        let abi = abi.ok_or_else(|| serde::de::Error::missing_field("abi"))?;
        Ok(AbiObject { abi, bytecode, deployed_bytecode })
    }
}

impl<'de> Deserialize<'de> for AbiObject {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(AbiObjectVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::Abi;

    fn assert_has_bytecode(s: &str) {
        match serde_json::from_str::<JsonAbi>(s).unwrap() {
            JsonAbi::Object(abi) => {
                assert!(abi.bytecode.is_some());
            }
            _ => {
                panic!("expected abi object")
            }
        }
    }

    #[test]
    fn can_parse_raw_abi() {
        const VERIFIER_ABI: &str =
            include_str!("../../../ethers-contract/tests/solidity-contracts/verifier_abi.json");
        let _ = serde_json::from_str::<RawAbi>(VERIFIER_ABI).unwrap();
    }

    #[test]
    fn can_parse_hardhat_raw_abi() {
        const VERIFIER_ABI: &str = include_str!(
            "../../../ethers-contract/tests/solidity-contracts/verifier_abi_hardhat.json"
        );
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

    #[test]
    fn can_deserialize_abi_object() {
        let abi_str = r#"[{"anonymous":false,"inputs":[{"indexed":true,"internalType":"uint64","name":"number","type":"uint64"}],"name":"MyEvent","type":"event"},{"inputs":[],"name":"greet","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
        let abi = serde_json::from_str::<JsonAbi>(abi_str).unwrap();
        assert!(matches!(abi, JsonAbi::Array(_)));

        let code = "0x608060405234801561001057600080fd5b50610242806100206000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c80635581701b14610030575b600080fd5b61004a60048036038101906100459190610199565b610060565b60405161005791906101f1565b60405180910390f35b610068610070565b819050919050565b60405180602001604052806000151581525090565b6000604051905090565b600080fd5b600080fd5b6000601f19601f8301169050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b6100e282610099565b810181811067ffffffffffffffff82111715610101576101006100aa565b5b80604052505050565b6000610114610085565b905061012082826100d9565b919050565b60008115159050919050565b61013a81610125565b811461014557600080fd5b50565b60008135905061015781610131565b92915050565b60006020828403121561017357610172610094565b5b61017d602061010a565b9050600061018d84828501610148565b60008301525092915050565b6000602082840312156101af576101ae61008f565b5b60006101bd8482850161015d565b91505092915050565b6101cf81610125565b82525050565b6020820160008201516101eb60008501826101c6565b50505050565b600060208201905061020660008301846101d5565b9291505056fea2646970667358221220890202b0964477379a457ab3725a21d7c14581e4596552e32a54e23f1c6564e064736f6c634300080c0033";
        let s = format!(r#"{{"abi": {abi_str}, "bin" : "{code}" }}"#);
        assert_has_bytecode(&s);

        let s = format!(r#"{{"abi": {abi_str}, "bytecode" : {{ "object": "{code}" }} }}"#);
        assert_has_bytecode(&s);

        let s = format!(r#"{{"abi": {abi_str}, "bytecode" : "{code}" }}"#);
        assert_has_bytecode(&s);

        let hh_artifact = include_str!(
            "../../../ethers-contract/tests/solidity-contracts/verifier_abi_hardhat.json"
        );
        match serde_json::from_str::<JsonAbi>(hh_artifact).unwrap() {
            JsonAbi::Object(abi) => {
                assert!(abi.bytecode.is_none());
            }
            _ => {
                panic!("expected abi object")
            }
        }
    }

    #[test]
    fn can_parse_greeter_bytecode() {
        let artifact =
            include_str!("../../../ethers-contract/tests/solidity-contracts/greeter.json");
        assert_has_bytecode(artifact);
    }

    #[test]
    fn ignores_empty_bytecode() {
        let abi_str = r#"[{"anonymous":false,"inputs":[{"indexed":true,"internalType":"uint64","name":"number","type":"uint64"}],"name":"MyEvent","type":"event"},{"inputs":[],"name":"greet","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#;
        let s = format!(r#"{{"abi": {abi_str}, "bin" : "0x" }}"#);

        match serde_json::from_str::<JsonAbi>(&s).unwrap() {
            JsonAbi::Object(abi) => {
                assert!(abi.bytecode.is_none());
            }
            _ => {
                panic!("expected abi object")
            }
        }

        let s = format!(r#"{{"abi": {abi_str}, "bytecode" : {{ "object": "0x" }} }}"#);

        match serde_json::from_str::<JsonAbi>(&s).unwrap() {
            JsonAbi::Object(abi) => {
                assert!(abi.bytecode.is_none());
            }
            _ => {
                panic!("expected abi object")
            }
        }
    }

    #[test]
    fn can_parse_deployed_bytecode() {
        let artifact = include_str!("../../testdata/solc-obj.json");
        match serde_json::from_str::<JsonAbi>(artifact).unwrap() {
            JsonAbi::Object(abi) => {
                assert!(abi.bytecode.is_some());
                assert!(abi.deployed_bytecode.is_some());
            }
            _ => {
                panic!("expected abi object")
            }
        }
    }
}
