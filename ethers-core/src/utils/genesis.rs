use std::collections::HashMap;

use crate::{
    types::{Address, Bytes, H256, U256, U64},
    utils::{from_int_or_hex, from_int_or_hex_opt},
};
use serde::{Deserialize, Serialize};

/// This represents the chain configuration, specifying the genesis block, header fields, and hard
/// fork switch blocks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// The fork configuration for this network.
    pub config: ChainConfig,

    /// The genesis header nonce.
    pub nonce: U64,

    /// The genesis header timestamp.
    pub timestamp: U64,

    /// The genesis header extra data.
    pub extra_data: Bytes,

    /// The genesis header gas limit.
    pub gas_limit: U64,

    /// The genesis header difficulty.
    #[serde(deserialize_with = "from_int_or_hex")]
    pub difficulty: U256,

    /// The genesis header mix hash.
    pub mix_hash: H256,

    /// The genesis header coinbase address.
    pub coinbase: Address,

    /// The initial state of the genesis block.
    pub alloc: HashMap<Address, GenesisAccount>,
}

impl Genesis {
    /// Creates a chain config using the given chain id.
    /// and funds the given address with max coins.
    ///
    /// Enables all hard forks up to London at genesis.
    pub fn new(chain_id: u64, signer_addr: Address) -> Genesis {
        // set up a clique config with an instant sealing period and short (8 block) epoch
        let clique_config = CliqueConfig { period: 0, epoch: 8 };

        let config = ChainConfig {
            chain_id,
            eip155_block: Some(0),
            eip150_block: Some(0),
            eip158_block: Some(0),

            homestead_block: Some(0),
            byzantium_block: Some(0),
            constantinople_block: Some(0),
            petersburg_block: Some(0),
            istanbul_block: Some(0),
            muir_glacier_block: Some(0),
            berlin_block: Some(0),
            london_block: Some(0),
            clique: Some(clique_config),
            ..Default::default()
        };

        // fund account
        let mut alloc = HashMap::new();
        alloc.insert(
            signer_addr,
            GenesisAccount { balance: U256::MAX, nonce: None, code: None, storage: None },
        );

        // put signer address in the extra data, padded by the required amount of zeros
        // Clique issue: https://github.com/ethereum/EIPs/issues/225
        // Clique EIP: https://eips.ethereum.org/EIPS/eip-225
        //
        // The first 32 bytes are vanity data, so we will populate it with zeros
        // This is followed by the signer address, which is 20 bytes
        // There are 65 bytes of zeros after the signer address, which is usually populated with the
        // proposer signature. Because the genesis does not have a proposer signature, it will be
        // populated with zeros.
        let extra_data_bytes = [&[0u8; 32][..], signer_addr.as_bytes(), &[0u8; 65][..]].concat();
        let extra_data = Bytes::from(extra_data_bytes);

        Genesis {
            config,
            alloc,
            difficulty: U256::one(),
            gas_limit: U64::from(5000000),
            extra_data,
            ..Default::default()
        }
    }
}

/// An account in the state of the genesis block.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenesisAccount {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    pub balance: U256,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<Bytes>,
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub storage: Option<HashMap<H256, H256>>,
}

/// Represents a node's chain configuration.
///
/// See [geth's `ChainConfig`
/// struct](https://github.com/ethereum/go-ethereum/blob/64dccf7aa411c5c7cd36090c3d9b9892945ae813/params/config.go#L349)
/// for the source of each field.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct ChainConfig {
    /// The network's chain ID.
    pub chain_id: u64,

    /// The homestead switch block (None = no fork, 0 = already homestead).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homestead_block: Option<u64>,

    /// The DAO fork switch block (None = no fork).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dao_fork_block: Option<u64>,

    /// Whether or not the node supports the DAO hard-fork.
    pub dao_fork_support: bool,

    /// The EIP-150 hard fork block (None = no fork).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip150_block: Option<u64>,

    /// The EIP-150 hard fork hash.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip150_hash: Option<H256>,

    /// The EIP-155 hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip155_block: Option<u64>,

    /// The EIP-158 hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eip158_block: Option<u64>,

    /// The Byzantium hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub byzantium_block: Option<u64>,

    /// The Constantinople hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constantinople_block: Option<u64>,

    /// The Petersburg hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub petersburg_block: Option<u64>,

    /// The Istanbul hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub istanbul_block: Option<u64>,

    /// The Muir Glacier hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub muir_glacier_block: Option<u64>,

    /// The Berlin hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub berlin_block: Option<u64>,

    /// The London hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub london_block: Option<u64>,

    /// The Arrow Glacier hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arrow_glacier_block: Option<u64>,

    /// The Gray Glacier hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gray_glacier_block: Option<u64>,

    /// Virtual fork after the merge to use as a network splitter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_netsplit_block: Option<u64>,

    /// Shanghai switch time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shanghai_time: Option<u64>,

    // TODO: change to cancunTime when <https://github.com/ethereum/go-ethereum/pull/26481> is
    // merged in geth
    /// Cancun hard fork block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cancun_block: Option<u64>,

    /// Total difficulty reached that triggers the merge consensus upgrade.
    #[serde(skip_serializing_if = "Option::is_none", deserialize_with = "from_int_or_hex_opt")]
    pub terminal_total_difficulty: Option<U256>,

    /// A flag specifying that the network already passed the terminal total difficulty. Its
    /// purpose is to disable legacy sync without having seen the TTD locally.
    pub terminal_total_difficulty_passed: bool,

    /// Ethash parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethash: Option<EthashConfig>,

    /// Clique parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clique: Option<CliqueConfig>,
}

/// Empty consensus configuration for proof-of-work networks.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthashConfig {}

/// Consensus configuration for Clique.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CliqueConfig {
    /// Number of seconds between blocks to enforce.
    pub period: u64,

    /// Epoch length to reset votes and checkpoints.
    pub epoch: u64,
}

#[cfg(test)]
mod tests {
    use super::Genesis;

    #[test]
    fn parse_hive_genesis() {
        let geth_genesis = r#"
        {
            "difficulty": "0x20000",
            "gasLimit": "0x1",
            "alloc": {},
            "config": {
              "ethash": {},
              "chainId": 1
            }
        }
        "#;

        let _genesis: Genesis = serde_json::from_str(geth_genesis).unwrap();
    }

    #[test]
    fn parse_hive_rpc_genesis() {
        let geth_genesis = r#"
        {
          "config": {
            "chainId": 7,
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip150Hash": "0x5de1ee4135274003348e80b788e5afa4b18b18d320a5622218d5c493fedf5689",
            "eip155Block": 0,
            "eip158Block": 0
          },
          "coinbase": "0x0000000000000000000000000000000000000000",
          "difficulty": "0x20000",
          "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000658bdf435d810c91414ec09147daa6db624063790000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
          "gasLimit": "0x2fefd8",
          "nonce": "0x0000000000000000",
          "timestamp": "0x1234",
          "alloc": {
            "cf49fda3be353c69b41ed96333cd24302da4556f": {
              "balance": "0x123450000000000000000"
            },
            "0161e041aad467a890839d5b08b138c1e6373072": {
              "balance": "0x123450000000000000000"
            },
            "87da6a8c6e9eff15d703fc2773e32f6af8dbe301": {
              "balance": "0x123450000000000000000"
            },
            "b97de4b8c857e4f6bc354f226dc3249aaee49209": {
              "balance": "0x123450000000000000000"
            },
            "c5065c9eeebe6df2c2284d046bfc906501846c51": {
              "balance": "0x123450000000000000000"
            },
            "0000000000000000000000000000000000000314": {
              "balance": "0x0",
              "code": "0x60606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063a223e05d1461006a578063abd1a0cf1461008d578063abfced1d146100d4578063e05c914a14610110578063e6768b451461014c575b610000565b346100005761007761019d565b6040518082815260200191505060405180910390f35b34610000576100be600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919050506101a3565b6040518082815260200191505060405180910390f35b346100005761010e600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919080359060200190919050506101ed565b005b346100005761014a600480803590602001909190803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610236565b005b346100005761017960048080359060200190919080359060200190919080359060200190919050506103c4565b60405180848152602001838152602001828152602001935050505060405180910390f35b60005481565b6000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490505b919050565b80600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055505b5050565b7f6031a8d62d7c95988fa262657cd92107d90ed96e08d8f867d32f26edfe85502260405180905060405180910390a17f47e2689743f14e97f7dcfa5eec10ba1dff02f83b3d1d4b9c07b206cbbda66450826040518082815260200191505060405180910390a1817fa48a6b249a5084126c3da369fbc9b16827ead8cb5cdc094b717d3f1dcd995e2960405180905060405180910390a27f7890603b316f3509577afd111710f9ebeefa15e12f72347d9dffd0d65ae3bade81604051808273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390a18073ffffffffffffffffffffffffffffffffffffffff167f7efef9ea3f60ddc038e50cccec621f86a0195894dc0520482abf8b5c6b659e4160405180905060405180910390a28181604051808381526020018273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019250505060405180910390a05b5050565b6000600060008585859250925092505b935093509390505600a165627a7a72305820aaf842d0d0c35c45622c5263cbb54813d2974d3999c8c38551d7c613ea2bc1170029",
              "storage": {
                "0x0000000000000000000000000000000000000000000000000000000000000000": "0x1234",
                "0x6661e9d6d8b923d5bbaab1b96e1dd51ff6ea2a93520fdc9eb75d059238b8c5e9": "0x01"
              }
            },
            "0000000000000000000000000000000000000315": {
              "balance": "0x9999999999999999999999999999999",
              "code": "0x60606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063ef2769ca1461003e575b610000565b3461000057610078600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803590602001909190505061007a565b005b8173ffffffffffffffffffffffffffffffffffffffff166108fc829081150290604051809050600060405180830381858888f1935050505015610106578173ffffffffffffffffffffffffffffffffffffffff167f30a3c50752f2552dcc2b93f5b96866280816a986c0c0408cb6778b9fa198288f826040518082815260200191505060405180910390a25b5b50505600a165627a7a72305820637991fabcc8abad4294bf2bb615db78fbec4edff1635a2647d3894e2daf6a610029"
            }
          }
        }
        "#;

        let _genesis: Genesis = serde_json::from_str(geth_genesis).unwrap();
    }

    #[test]
    fn parse_hive_graphql_genesis() {
        let geth_genesis = r#"
        {
            "config"     : {},
            "coinbase"   : "0x8888f1f195afa192cfee860698584c030f4c9db1",
            "difficulty" : "0x020000",
            "extraData"  : "0x42",
            "gasLimit"   : "0x2fefd8",
            "mixHash"    : "0x2c85bcbce56429100b2108254bb56906257582aeafcbd682bc9af67a9f5aee46",
            "nonce"      : "0x78cc16f7b4f65485",
            "parentHash" : "0x0000000000000000000000000000000000000000000000000000000000000000",
            "timestamp"  : "0x54c98c81",
            "alloc"      : {
                "a94f5374fce5edbc8e2a8697c15331677e6ebf0b": {
                    "balance" : "0x09184e72a000"
                }
            }
        }
        "#;

        let _genesis: Genesis = serde_json::from_str(geth_genesis).unwrap();
    }

    #[test]
    fn parse_hive_engine_genesis() {
        let geth_genesis = r#"
        {
          "config": {
            "chainId": 7,
            "homesteadBlock": 0,
            "eip150Block": 0,
            "eip150Hash": "0x5de1ee4135274003348e80b788e5afa4b18b18d320a5622218d5c493fedf5689",
            "eip155Block": 0,
            "eip158Block": 0,
            "byzantiumBlock": 0,
            "constantinopleBlock": 0,
            "petersburgBlock": 0,
            "istanbulBlock": 0,
            "muirGlacierBlock": 0,
            "berlinBlock": 0,
            "yolov2Block": 0,
            "yolov3Block": 0,
            "londonBlock": 0
          },
          "coinbase": "0x0000000000000000000000000000000000000000",
          "difficulty": "0x30000",
          "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000658bdf435d810c91414ec09147daa6db624063790000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
          "gasLimit": "0x2fefd8",
          "nonce": "0x0000000000000000",
          "timestamp": "0x1234",
          "alloc": {
            "cf49fda3be353c69b41ed96333cd24302da4556f": {
              "balance": "0x123450000000000000000"
            },
            "0161e041aad467a890839d5b08b138c1e6373072": {
              "balance": "0x123450000000000000000"
            },
            "87da6a8c6e9eff15d703fc2773e32f6af8dbe301": {
              "balance": "0x123450000000000000000"
            },
            "b97de4b8c857e4f6bc354f226dc3249aaee49209": {
              "balance": "0x123450000000000000000"
            },
            "c5065c9eeebe6df2c2284d046bfc906501846c51": {
              "balance": "0x123450000000000000000"
            },
            "0000000000000000000000000000000000000314": {
              "balance": "0x0",
              "code": "0x60606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063a223e05d1461006a578063abd1a0cf1461008d578063abfced1d146100d4578063e05c914a14610110578063e6768b451461014c575b610000565b346100005761007761019d565b6040518082815260200191505060405180910390f35b34610000576100be600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919050506101a3565b6040518082815260200191505060405180910390f35b346100005761010e600480803573ffffffffffffffffffffffffffffffffffffffff169060200190919080359060200190919050506101ed565b005b346100005761014a600480803590602001909190803573ffffffffffffffffffffffffffffffffffffffff16906020019091905050610236565b005b346100005761017960048080359060200190919080359060200190919080359060200190919050506103c4565b60405180848152602001838152602001828152602001935050505060405180910390f35b60005481565b6000600160008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081526020016000205490505b919050565b80600160008473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055505b5050565b7f6031a8d62d7c95988fa262657cd92107d90ed96e08d8f867d32f26edfe85502260405180905060405180910390a17f47e2689743f14e97f7dcfa5eec10ba1dff02f83b3d1d4b9c07b206cbbda66450826040518082815260200191505060405180910390a1817fa48a6b249a5084126c3da369fbc9b16827ead8cb5cdc094b717d3f1dcd995e2960405180905060405180910390a27f7890603b316f3509577afd111710f9ebeefa15e12f72347d9dffd0d65ae3bade81604051808273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200191505060405180910390a18073ffffffffffffffffffffffffffffffffffffffff167f7efef9ea3f60ddc038e50cccec621f86a0195894dc0520482abf8b5c6b659e4160405180905060405180910390a28181604051808381526020018273ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019250505060405180910390a05b5050565b6000600060008585859250925092505b935093509390505600a165627a7a72305820aaf842d0d0c35c45622c5263cbb54813d2974d3999c8c38551d7c613ea2bc1170029",
              "storage": {
                "0x0000000000000000000000000000000000000000000000000000000000000000": "0x1234",
                "0x6661e9d6d8b923d5bbaab1b96e1dd51ff6ea2a93520fdc9eb75d059238b8c5e9": "0x01"
              }
            },
            "0000000000000000000000000000000000000315": {
              "balance": "0x9999999999999999999999999999999",
              "code": "0x60606040526000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063ef2769ca1461003e575b610000565b3461000057610078600480803573ffffffffffffffffffffffffffffffffffffffff1690602001909190803590602001909190505061007a565b005b8173ffffffffffffffffffffffffffffffffffffffff166108fc829081150290604051809050600060405180830381858888f1935050505015610106578173ffffffffffffffffffffffffffffffffffffffff167f30a3c50752f2552dcc2b93f5b96866280816a986c0c0408cb6778b9fa198288f826040518082815260200191505060405180910390a25b5b50505600a165627a7a72305820637991fabcc8abad4294bf2bb615db78fbec4edff1635a2647d3894e2daf6a610029"
            },
            "0000000000000000000000000000000000000316": {
              "balance": "0x0",
              "code": "0x444355"
            },
            "0000000000000000000000000000000000000317": {
              "balance": "0x0",
              "code": "0x600160003555"
            }
          }
        }
        "#;

        let _genesis: Genesis = serde_json::from_str(geth_genesis).unwrap();
    }

    #[test]
    fn parse_hive_devp2p_genesis() {
        let geth_genesis = r#"
        {
            "config": {
                "chainId": 19763,
                "homesteadBlock": 0,
                "eip150Block": 0,
                "eip155Block": 0,
                "eip158Block": 0,
                "byzantiumBlock": 0,
                "ethash": {}
            },
            "nonce": "0xdeadbeefdeadbeef",
            "timestamp": "0x0",
            "extraData": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "gasLimit": "0x80000000",
            "difficulty": "0x20000",
            "mixHash": "0x0000000000000000000000000000000000000000000000000000000000000000",
            "coinbase": "0x0000000000000000000000000000000000000000",
            "alloc": {
                "71562b71999873db5b286df957af199ec94617f7": {
                    "balance": "0xffffffffffffffffffffffffff"
                }
            },
            "number": "0x0",
            "gasUsed": "0x0",
            "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
        }
        "#;

        let _genesis: Genesis = serde_json::from_str(geth_genesis).unwrap();
    }
}
