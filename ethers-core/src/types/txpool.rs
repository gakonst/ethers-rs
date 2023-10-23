use crate::types::{Address, Transaction, U256, U64};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize, Serialize,
};
use std::{collections::BTreeMap, fmt, str::FromStr};

/// Transaction summary as found in the Txpool Inspection property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxpoolInspectSummary {
    /// Recipient (None when contract creation)
    pub to: Option<Address>,
    /// Transferred value
    pub value: U256,
    /// Gas amount
    pub gas: U256,
    /// Gas Price
    pub gas_price: U256,
}

/// Visitor struct for TxpoolInspectSummary.
struct TxpoolInspectSummaryVisitor;

/// Walk through the deserializer to parse a txpool inspection summary into the
/// `TxpoolInspectSummary` struct.
impl<'de> Visitor<'de> for TxpoolInspectSummaryVisitor {
    type Value = TxpoolInspectSummary;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("to: value wei + gasLimit gas × gas_price wei")
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let addr_split: Vec<&str> = value.split(": ").collect();
        if addr_split.len() != 2 {
            return Err(de::Error::custom("invalid format for TxpoolInspectSummary: to"))
        }
        let value_split: Vec<&str> = addr_split[1].split(" wei + ").collect();
        if value_split.len() != 2 {
            return Err(de::Error::custom("invalid format for TxpoolInspectSummary: gasLimit"))
        }
        let gas_split: Vec<&str> = value_split[1].split(" gas × ").collect();
        if gas_split.len() != 2 {
            return Err(de::Error::custom("invalid format for TxpoolInspectSummary: gas"))
        }
        let gas_price_split: Vec<&str> = gas_split[1].split(" wei").collect();
        if gas_price_split.len() != 2 {
            return Err(de::Error::custom("invalid format for TxpoolInspectSummary: gas_price"))
        }
        let addr = match addr_split[0] {
            "" => None,
            "0x" => None,
            "contract creation" => None,
            addr => {
                Some(Address::from_str(addr.trim_start_matches("0x")).map_err(de::Error::custom)?)
            }
        };
        let value = U256::from_dec_str(value_split[0]).map_err(de::Error::custom)?;
        let gas = U256::from_dec_str(gas_split[0]).map_err(de::Error::custom)?;
        let gas_price = U256::from_dec_str(gas_price_split[0]).map_err(de::Error::custom)?;

        Ok(TxpoolInspectSummary { to: addr, value, gas, gas_price })
    }
}

/// Implement the `Deserialize` trait for `TxpoolInspectSummary` struct.
impl<'de> Deserialize<'de> for TxpoolInspectSummary {
    fn deserialize<D>(deserializer: D) -> Result<TxpoolInspectSummary, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TxpoolInspectSummaryVisitor)
    }
}

/// Implement the `Serialize` trait for `TxpoolInspectSummary` struct so that the
/// format matches the one from geth.
impl Serialize for TxpoolInspectSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let formatted_to = if let Some(to) = self.to {
            format!("{to:?}")
        } else {
            "contract creation".to_string()
        };
        let formatted = format!(
            "{}: {} wei + {} gas × {} wei",
            formatted_to, self.value, self.gas, self.gas_price
        );
        serializer.serialize_str(&formatted)
    }
}

/// Transaction Pool Content
///
/// The content inspection property can be queried to list the exact details of all
/// the transactions currently pending for inclusion in the next block(s), as well
/// as the ones that are being scheduled for future execution only.
///
/// See [here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_content) for more details
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxpoolContent {
    /// pending tx
    pub pending: BTreeMap<Address, BTreeMap<String, Transaction>>,
    /// queued tx
    pub queued: BTreeMap<Address, BTreeMap<String, Transaction>>,
}

/// Transaction Pool Inspect
///
/// The inspect inspection property can be queried to list a textual summary
/// of all the transactions currently pending for inclusion in the next block(s),
/// as well as the ones that are being scheduled for future execution only.
/// This is a method specifically tailored to developers to quickly see the
/// transactions in the pool and find any potential issues.
///
/// See [here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_inspect) for more details
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxpoolInspect {
    /// pending tx
    pub pending: BTreeMap<Address, BTreeMap<String, TxpoolInspectSummary>>,
    /// queued tx
    pub queued: BTreeMap<Address, BTreeMap<String, TxpoolInspectSummary>>,
}

/// Transaction Pool Status
///
/// The status inspection property can be queried for the number of transactions
/// currently pending for inclusion in the next block(s), as well as the ones that
/// are being scheduled for future execution only.
///
/// See [here](https://geth.ethereum.org/docs/rpc/ns-txpool#txpool_status) for more details
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TxpoolStatus {
    /// number of pending tx
    pub pending: U64,
    /// number of queued tx
    pub queued: U64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_txpool_inspect() {
        let txpool_inspect_json = r#"
{
  "pending": {
    "0x0512261a7486b1e29704ac49a5eb355b6fd86872": {
      "124930": "0x000000000000000000000000000000000000007E: 0 wei + 100187 gas × 20000000000 wei"
    },
    "0x201354729f8d0f8b64e9a0c353c672c6a66b3857": {
      "252350": "0xd10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF: 0 wei + 65792 gas × 2000000000 wei",
      "252351": "0xd10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF: 0 wei + 65792 gas × 2000000000 wei",
      "252352": "0xd10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF: 0 wei + 65780 gas × 2000000000 wei",
      "252353": "0xd10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF: 0 wei + 65780 gas × 2000000000 wei"
    },
    "0x00000000863B56a3C1f0F1be8BC4F8b7BD78F57a": {
      "40": "contract creation: 0 wei + 612412 gas × 6000000000 wei"
    }
  },
  "queued": {
    "0x0f87ffcd71859233eb259f42b236c8e9873444e3": {
      "7": "0x3479BE69e07E838D9738a301Bb0c89e8EA2Bef4a: 1000000000000000 wei + 21000 gas × 10000000000 wei",
      "8": "0x73Aaf691bc33fe38f86260338EF88f9897eCaa4F: 1000000000000000 wei + 21000 gas × 10000000000 wei"
    },
    "0x307e8f249bcccfa5b245449256c5d7e6e079943e": {
      "3": "0x73Aaf691bc33fe38f86260338EF88f9897eCaa4F: 10000000000000000 wei + 21000 gas × 10000000000 wei"
    }
  }
}"#;
        let deserialized: TxpoolInspect = serde_json::from_str(txpool_inspect_json).unwrap();
        assert_eq!(deserialized, expected_txpool_inspect());

        let serialized = serde_json::to_string(&deserialized).unwrap();
        let deserialized2: TxpoolInspect = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized2, deserialized);
    }

    #[test]
    fn serde_txpool_status() {
        let txpool_status_json = r#"
{
  "pending": "0x23",
  "queued": "0x20"
}"#;
        let deserialized: TxpoolStatus = serde_json::from_str(txpool_status_json).unwrap();
        let serialized: String = serde_json::to_string_pretty(&deserialized).unwrap();
        assert_eq!(txpool_status_json.trim(), serialized);
    }

    fn expected_txpool_inspect() -> TxpoolInspect {
        let mut pending_map = BTreeMap::new();
        let mut pending_map_inner = BTreeMap::new();
        pending_map_inner.insert(
            "124930".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("000000000000000000000000000000000000007E").unwrap()),
                value: U256::from(0u128),
                gas: U256::from(100187u128),
                gas_price: U256::from(20000000000u128),
            },
        );
        pending_map.insert(
            Address::from_str("0512261a7486b1e29704ac49a5eb355b6fd86872").unwrap(),
            pending_map_inner.clone(),
        );
        pending_map_inner.clear();
        pending_map_inner.insert(
            "252350".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("d10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF").unwrap()),
                value: U256::from(0u128),
                gas: U256::from(65792u128),
                gas_price: U256::from(2000000000u128),
            },
        );
        pending_map_inner.insert(
            "252351".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("d10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF").unwrap()),
                value: U256::from(0u128),
                gas: U256::from(65792u128),
                gas_price: U256::from(2000000000u128),
            },
        );
        pending_map_inner.insert(
            "252352".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("d10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF").unwrap()),
                value: U256::from(0u128),
                gas: U256::from(65780u128),
                gas_price: U256::from(2000000000u128),
            },
        );
        pending_map_inner.insert(
            "252353".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("d10e3Be2bc8f959Bc8C41CF65F60dE721cF89ADF").unwrap()),
                value: U256::from(0u128),
                gas: U256::from(65780u128),
                gas_price: U256::from(2000000000u128),
            },
        );
        pending_map.insert(
            Address::from_str("201354729f8d0f8b64e9a0c353c672c6a66b3857").unwrap(),
            pending_map_inner.clone(),
        );
        pending_map_inner.clear();
        pending_map_inner.insert(
            "40".to_string(),
            TxpoolInspectSummary {
                to: None,
                value: U256::from(0u128),
                gas: U256::from(612412u128),
                gas_price: U256::from(6000000000u128),
            },
        );
        pending_map.insert(
            Address::from_str("00000000863B56a3C1f0F1be8BC4F8b7BD78F57a").unwrap(),
            pending_map_inner,
        );
        let mut queued_map = BTreeMap::new();
        let mut queued_map_inner = BTreeMap::new();
        queued_map_inner.insert(
            "7".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("3479BE69e07E838D9738a301Bb0c89e8EA2Bef4a").unwrap()),
                value: U256::from(1000000000000000u128),
                gas: U256::from(21000u128),
                gas_price: U256::from(10000000000u128),
            },
        );
        queued_map_inner.insert(
            "8".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("73Aaf691bc33fe38f86260338EF88f9897eCaa4F").unwrap()),
                value: U256::from(1000000000000000u128),
                gas: U256::from(21000u128),
                gas_price: U256::from(10000000000u128),
            },
        );
        queued_map.insert(
            Address::from_str("0f87ffcd71859233eb259f42b236c8e9873444e3").unwrap(),
            queued_map_inner.clone(),
        );
        queued_map_inner.clear();
        queued_map_inner.insert(
            "3".to_string(),
            TxpoolInspectSummary {
                to: Some(Address::from_str("73Aaf691bc33fe38f86260338EF88f9897eCaa4F").unwrap()),
                value: U256::from(10000000000000000u128),
                gas: U256::from(21000u128),
                gas_price: U256::from(10000000000u128),
            },
        );
        queued_map.insert(
            Address::from_str("307e8f249bcccfa5b245449256c5d7e6e079943e").unwrap(),
            queued_map_inner,
        );

        TxpoolInspect { pending: pending_map, queued: queued_map }
    }
}
