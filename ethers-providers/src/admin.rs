use crate::{H256, U256};
use enr::{k256::ecdsa::SigningKey, Enr};
use ethers_core::utils::{from_int_or_hex, ChainConfig};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

/// This includes general information about a running node, spanning networking and protocol
/// details.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeInfo {
    /// The node's secp256k1 public key.
    pub id: H256,

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

/// Represents a short summary of information known about a connected peer.
///
/// See [geth's `PeerInfo` struct](https://github.com/ethereum/go-ethereum/blob/64dccf7aa411c5c7cd36090c3d9b9892945ae813/p2p/peer.go#L484) for the source of each field.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    /// The peer's ENR.
    #[serde(default)]
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
#[serde(rename_all = "camelCase")]
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
