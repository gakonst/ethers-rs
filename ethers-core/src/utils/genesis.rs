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
