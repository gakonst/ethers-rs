use crate::types::{H256, U256};
use serde::{Deserialize, Serialize};

/// This represents the chain configuration, specifying the genesis block, header fields, and hard
/// fork switch blocks.
pub struct Genesis {}

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
    pub homestead_block: Option<u64>,

    /// The DAO fork switch block (None = no fork).
    pub dao_fork_block: Option<u64>,

    /// Whether or not the node supports the DAO hard-fork.
    pub dao_fork_support: bool,

    /// The EIP-150 hard fork block (None = no fork).
    pub eip150_block: Option<u64>,

    /// The EIP-150 hard fork hash.
    pub eip150_hash: Option<H256>,

    /// The EIP-155 hard fork block.
    pub eip155_block: Option<u64>,

    /// The EIP-158 hard fork block.
    pub eip158_block: Option<u64>,

    /// The Byzantium hard fork block.
    pub byzantium_block: Option<u64>,

    /// The Constantinople hard fork block.
    pub constantinople_block: Option<u64>,

    /// The Petersburg hard fork block.
    pub petersburg_block: Option<u64>,

    /// The Istanbul hard fork block.
    pub istanbul_block: Option<u64>,

    /// The Muir Glacier hard fork block.
    pub muir_glacier_block: Option<u64>,

    /// The Berlin hard fork block.
    pub berlin_block: Option<u64>,

    /// The London hard fork block.
    pub london_block: Option<u64>,

    /// The Arrow Glacier hard fork block.
    pub arrow_glacier_block: Option<u64>,

    /// The Gray Glacier hard fork block.
    pub gray_glacier_block: Option<u64>,

    /// Virtual fork after the merge to use as a network splitter.
    pub merge_netsplit_block: Option<u64>,

    /// The Shanghai hard fork block.
    pub shanghai_block: Option<u64>,

    /// The Cancun hard fork block.
    pub cancun_block: Option<u64>,

    /// Total difficulty reached that triggers the merge consensus upgrade.
    pub terminal_total_difficulty: Option<U256>,

    /// A flag specifying that the network already passed the terminal total difficulty. Its
    /// purpose is to disable legacy sync without having seen the TTD locally.
    pub terminal_total_difficulty_passed: bool,

    /// Ethash parameters.
    pub ethash: Option<EthashConfig>,

    /// Clique parameters.
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

