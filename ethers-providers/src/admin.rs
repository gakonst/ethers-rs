use crate::{H256, U256};
use enr::{k256::ecdsa::SigningKey, Enr};
use ethers_core::utils::from_int_or_hex;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

/// This includes general information about a running node, spanning networking and protocol
/// details.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeInfo {
    /// TODO: docs - what kind of key is this?
    pub id: String,

    /// The client user agent, containing a client name, version, OS, and other metadata.
    pub name: String,

    /// The enode URL of the running client.
    pub enode: String,

    /// The [ENR](https://eips.ethereum.org/EIPS/eip-778) of the running client.
    pub enr: Enr<SigningKey>,

    /// The IP address of the running client.
    pub ip: IpAddr,

    /// The client's listening ports.
    pub ports: Ports,

    /// The client's listening address.
    #[serde(rename = "listenAddr")]
    pub listen_addr: String,

    /// The protocols that the client supports, with protocol metadata.
    pub protocols: ProtocolInfo,
}

/// Represents a node's discovery and listener ports.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ports {
    /// The discovery port of the running client.
    pub discovery: u16,

    /// The listener port of the running client.
    pub listener: u16,
}

/// Represents the protocols that the client supports.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolInfo {
    pub eth: Option<EthProtocolInfo>,
    pub snap: Option<SnapProtocolInfo>,
}

/// Represents a short summary of the `eth` sub-protocol metadata known about the host peer.
///
/// See [geth's `NodeInfo`
/// struct](https://github.com/ethereum/go-ethereum/blob/c2e0abce2eedc1ba2a1b32c46fd07ef18a25354a/eth/protocols/eth/handler.go#L129)
/// for how these fields are determined.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthProtocolInfo {
    /// The ethereum network ID.
    pub network: u64,

    /// The total difficulty of the host's blockchain.
    #[serde(deserialize_with = "from_int_or_hex")]
    pub difficulty: U256,

    /// The Keccak hash of the host's genesis block.
    pub genesis: H256,

    /// The chain configuration for the host's fork rules.
    pub config: ChainConfig,

    /// The hash of the host's best known block.
    pub head: H256,
}

/// Represents a short summary of the `snap` sub-protocol metadata known about the host peer.
///
/// This is just an empty struct, because [geth's internal representation is
/// empty](https://github.com/ethereum/go-ethereum/blob/c2e0abce2eedc1ba2a1b32c46fd07ef18a25354a/eth/protocols/snap/handler.go#L571-L576).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapProtocolInfo {}

/// Represents a node's chain configuration.
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default)]
pub struct ChainConfig {
    /// The network's chain ID.
    #[serde(rename = "chainId")]
    pub chain_id: u64,

    /// The homestead switch block (None = no fork, 0 = already homestead).
    #[serde(rename = "homesteadBlock")]
    pub homestead_block: Option<u64>,

    /// The DAO fork switch block (None = no fork).
    #[serde(rename = "daoForkBlock")]
    pub dao_fork_block: Option<u64>,

    /// Whether or not the node supports the DAO hard-fork.
    #[serde(rename = "daoForkSupport")]
    pub dao_fork_support: bool,

    /// The EIP-150 hard fork block (None = no fork).
    #[serde(rename = "eip150Block")]
    pub eip150_block: Option<u64>,

    /// The EIP-150 hard fork hash.
    #[serde(rename = "eip150Hash")]
    pub eip150_hash: Option<H256>,

    /// The EIP-155 hard fork block.
    #[serde(rename = "eip155Block")]
    pub eip155_block: Option<u64>,

    /// The EIP-158 hard fork block.
    #[serde(rename = "eip158Block")]
    pub eip158_block: Option<u64>,

    /// The Byzantium hard fork block.
    #[serde(rename = "byzantiumBlock")]
    pub byzantium_block: Option<u64>,

    /// The Constantinople hard fork block.
    #[serde(rename = "constantinopleBlock")]
    pub constantinople_block: Option<u64>,

    /// The Petersburg hard fork block.
    #[serde(rename = "petersburgBlock")]
    pub petersburg_block: Option<u64>,

    /// The Istanbul hard fork block.
    #[serde(rename = "istanbulBlock")]
    pub istanbul_block: Option<u64>,

    /// The Muir Glacier hard fork block.
    #[serde(rename = "muirGlacierBlock")]
    pub muir_glacier_block: Option<u64>,

    /// The Berlin hard fork block.
    #[serde(rename = "berlinBlock")]
    pub berlin_block: Option<u64>,

    /// The London hard fork block.
    #[serde(rename = "londonBlock")]
    pub london_block: Option<u64>,

    /// The Arrow Glacier hard fork block.
    #[serde(rename = "arrowGlacierBlock")]
    pub arrow_glacier_block: Option<u64>,

    /// The Gray Glacier hard fork block.
    #[serde(rename = "grayGlacierBlock")]
    pub gray_glacier_block: Option<u64>,

    /// Virtual fork after the merge to use as a network splitter.
    #[serde(rename = "mergeNetsplitBlock")]
    pub merge_netsplit_block: Option<u64>,

    /// The Shanghai hard fork block.
    #[serde(rename = "shanghaiBlock")]
    pub shanghai_block: Option<u64>,

    /// The Cancun hard fork block.
    #[serde(rename = "cancunBlock")]
    pub cancun_block: Option<u64>,

    /// Total difficulty reached that triggers the merge consensus upgrade.
    #[serde(rename = "terminalTotalDifficulty")]
    pub terminal_total_difficulty: Option<U256>,

    /// A flag specifying that the network already passed the terminal total difficulty. Its
    /// purpose is to disable legacy sync without having seen the TTD locally.
    #[serde(rename = "terminalTotalDifficultyPassed")]
    pub terminal_total_difficulty_passed: bool,

    /// Ethash parameters.
    pub ethash: Option<EthashConfig>,

    /// Clique parameters.
    pub clique: Option<CliqueConfig>,
}

/// Represents a short summary of information known about a connected peer.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    /// The peer's ENR.
    pub enr: Option<Enr<SigningKey>>,

    /// The peer's enode URL.
    pub enode: String,

    /// The peer's network ID.
    pub id: String,

    /// The peer's name.
    pub name: String,

    /// The peer's capabilities.
    pub caps: Vec<String>,

    /// Networking information about the peer.
    pub network: PeerNetworkInfo,

    /// The protocols that the peer supports, with protocol metadata.
    pub protocols: Vec<ProtocolInfo>,
}

/// Represents networking related information about the peer, including details about whether or
/// not it is inbound, trusted, or static.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerNetworkInfo {
    /// The local endpoint of the TCP connection.
    pub local_address: SocketAddr,

    /// The remote endpoint of the TCP connection.
    pub remote_address: SocketAddr,

    /// Whether or not the peer is inbound.
    pub inbound: bool,

    /// Whether or not the peer is trusted.
    pub trusted: bool,

    /// Whether or not the peer is a static peer.
    pub static_node: bool,
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

/// An event emitted by geth when a peer is added or removed, or when a message is sent or
/// received.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerEvent {
    /// The type of event.
    #[serde(rename = "type")]
    pub event_type: String,

    /// The peer's enode ID.
    pub peer: String,

    /// The error associated with the event.
    pub error: Option<String>,

    /// The protocol associated with the event.
    pub protocol: Option<String>,

    /// The message code.
    pub msg_code: Option<u64>,

    /// The message size.
    pub msg_size: Option<u64>,

    /// The local address of the peer.
    pub local_address: Option<SocketAddr>,

    /// The remote address of the peer.
    pub remote_address: Option<SocketAddr>,
}
