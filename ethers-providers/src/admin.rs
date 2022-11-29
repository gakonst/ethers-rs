use crate::{H256, U256};
use enr::{k256::ecdsa::SigningKey, Enr};
use ethers_core::utils::{from_int_or_hex, ChainConfig};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};

/// This includes general information about a running node, spanning networking and protocol
/// details.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NodeInfo {
    /// The node's private key.
    pub id: H256,

    /// The node's user agent, containing a client name, version, OS, and other metadata.
    pub name: String,

    /// The enode URL of the connected node.
    pub enode: String,

    /// The [ENR](https://eips.ethereum.org/EIPS/eip-778) of the running client.
    pub enr: Enr<SigningKey>,

    /// The IP address of the connected node.
    pub ip: IpAddr,

    /// The node's listening ports.
    pub ports: Ports,

    /// The node's listening address.
    #[serde(rename = "listenAddr")]
    pub listen_addr: String,

    /// The protocols that the node supports, with protocol metadata.
    pub protocols: ProtocolInfo,
}

/// Represents a node's discovery and listener ports.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ports {
    /// The node's discovery port.
    pub discovery: u16,

    /// The node's listener port.
    pub listener: u16,
}

/// Represents protocols that the connected RPC node supports.
///
/// This contains protocol information reported by the connected RPC node.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProtocolInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eth: Option<EthProtocolInfo>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snap: Option<SnapProtocolInfo>,
}

/// Represents a short summary of the `eth` sub-protocol metadata known about the host peer.
///
/// See [geth's `NodeInfo`
/// struct](https://github.com/ethereum/go-ethereum/blob/c2e0abce2eedc1ba2a1b32c46fd07ef18a25354a/eth/protocols/eth/handler.go#L129)
/// for how these fields are determined.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EthProtocolInfo {
    /// The eth network version.
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

/// Represents a short summary of the host's `snap` sub-protocol metadata.
///
/// This is just an empty struct, because [geth's internal representation is
/// empty](https://github.com/ethereum/go-ethereum/blob/c2e0abce2eedc1ba2a1b32c46fd07ef18a25354a/eth/protocols/snap/handler.go#L571-L576).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SnapProtocolInfo {}

/// Represents the protocols that a peer supports.
///
/// This differs from [`ProtocolInfo`] in that [`PeerProtocolInfo`] contains protocol information
/// gathered from the protocol handshake, and [`ProtocolInfo`] contains information reported by the
/// connected RPC node.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerProtocolInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eth: Option<EthPeerInfo>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub snap: Option<SnapPeerInfo>,
}

/// Can contain either eth protocol info or a string "handshake", which geth uses if the peer is
/// still completing the handshake for the protocol.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum EthPeerInfo {
    /// The `eth` sub-protocol metadata known about the host peer.
    Info(Box<EthInfo>),

    /// The string "handshake" if the peer is still completing the handshake for the protocol.
    #[serde(deserialize_with = "deser_handshake", serialize_with = "ser_handshake")]
    Handshake,
}

/// Represents a short summary of the `eth` sub-protocol metadata known about a connected peer
///
/// See [geth's `ethPeerInfo`
/// struct](https://github.com/ethereum/go-ethereum/blob/53d1ae096ac0515173e17f0f81a553e5f39027f7/eth/peer.go#L28)
/// for how these fields are determined.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct EthInfo {
    /// The negotiated eth version.
    pub version: u64,

    /// The total difficulty of the peer's blockchain.
    #[serde(deserialize_with = "from_int_or_hex")]
    pub difficulty: U256,

    /// The hash of the peer's best known block.
    pub head: H256,
}

/// Can contain either snap protocol info or a string "handshake", which geth uses if the peer is
/// still completing the handshake for the protocol.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum SnapPeerInfo {
    /// The `snap` sub-protocol metadata known about the host peer.
    Info(SnapInfo),

    /// The string "handshake" if the peer is still completing the handshake for the protocol.
    #[serde(deserialize_with = "deser_handshake", serialize_with = "ser_handshake")]
    Handshake,
}

/// Represents a short summary of the `snap` sub-protocol metadata known about a connected peer.
///
/// See [geth's `snapPeerInfo`
/// struct](https://github.com/ethereum/go-ethereum/blob/53d1ae096ac0515173e17f0f81a553e5f39027f7/eth/peer.go#L53)
/// for how these fields are determined.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SnapInfo {
    /// The negotiated snap version.
    pub version: u64,
}

/// Represents a short summary of information known about a connected peer.
///
/// See [geth's `PeerInfo` struct](https://github.com/ethereum/go-ethereum/blob/64dccf7aa411c5c7cd36090c3d9b9892945ae813/p2p/peer.go#L484) for the source of each field.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerInfo {
    /// The peer's ENR.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enr: Option<Enr<SigningKey>>,

    /// The peer's enode URL.
    pub enode: String,

    /// The peer's enode ID.
    pub id: String,

    /// The peer's name.
    pub name: String,

    /// The peer's capabilities.
    pub caps: Vec<String>,

    /// Networking information about the peer.
    pub network: PeerNetworkInfo,

    /// The protocols that the peer supports, with protocol metadata.
    pub protocols: PeerProtocolInfo,
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
    #[serde(rename = "static")]
    pub static_node: bool,
}

fn deser_handshake<'de, D>(deserializer: D) -> Result<(), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s == "handshake" {
        Ok(())
    } else {
        Err(serde::de::Error::custom(
            "expected \"handshake\" if protocol info did not appear in the response",
        ))
    }
}

fn ser_handshake<S>(serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str("handshake")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_peer_info() {
        let response = r#"{
            "enode":"enode://bb37b7302f79e47c1226d6e3ccf0ef6d51146019efdcc1f6e861fd1c1a78d5e84e486225a6a8a503b93d5c50125ee980835c92bde7f7d12f074c16f4e439a578@127.0.0.1:60872",
            "id":"ca23c04b7e796da5d6a5f04a62b81c88d41b1341537db85a2b6443e838d8339b",
            "name":"Geth/v1.10.19-stable/darwin-arm64/go1.18.3",
            "caps":["eth/66","eth/67","snap/1"],
            "network":{
                "localAddress":"127.0.0.1:30304",
                "remoteAddress":"127.0.0.1:60872",
                "inbound":true,
                "trusted":false,
                "static":false
            },
            "protocols":{
                "eth":{
                    "version":67,
                    "difficulty":0,
                    "head":"0xb04009ddf4b0763f42778e7d5937e49bebf1e11b2d26c9dac6cefb5f84b6f8ea"
                },
                "snap":{"version":1}
            }
        }"#;
        let peer_info: PeerInfo = serde_json::from_str(response).unwrap();

        assert_eq!(peer_info.enode, "enode://bb37b7302f79e47c1226d6e3ccf0ef6d51146019efdcc1f6e861fd1c1a78d5e84e486225a6a8a503b93d5c50125ee980835c92bde7f7d12f074c16f4e439a578@127.0.0.1:60872");
    }

    #[test]
    fn deserialize_node_info() {
        // this response also has an enr
        let response = r#"{
            "id":"6e2fe698f3064cd99410926ce16734e35e3cc947d4354461d2594f2d2dd9f7b6",
            "name":"Geth/v1.10.19-stable/darwin-arm64/go1.18.3",
            "enode":"enode://d7dfaea49c7ef37701e668652bcf1bc63d3abb2ae97593374a949e175e4ff128730a2f35199f3462a56298b981dfc395a5abebd2d6f0284ffe5bdc3d8e258b86@127.0.0.1:30304?discport=0",
            "enr":"enr:-Jy4QIvS0dKBLjTTV_RojS8hjriwWsJNHRVyOh4Pk4aUXc5SZjKRVIOeYc7BqzEmbCjLdIY4Ln7x5ZPf-2SsBAc2_zqGAYSwY1zog2V0aMfGhNegsXuAgmlkgnY0gmlwhBiT_DiJc2VjcDI1NmsxoQLX366knH7zdwHmaGUrzxvGPTq7Kul1kzdKlJ4XXk_xKIRzbmFwwIN0Y3CCdmA",
            "ip":"127.0.0.1",
            "ports":{
                "discovery":0,
                "listener":30304
            },
            "listenAddr":"[::]:30304",
            "protocols":{
                "eth":{
                    "network":1337,
                    "difficulty":0,
                    "genesis":"0xb04009ddf4b0763f42778e7d5937e49bebf1e11b2d26c9dac6cefb5f84b6f8ea",
                    "config":{
                        "chainId":0,
                        "eip150Hash":"0x0000000000000000000000000000000000000000000000000000000000000000"
                    },
                    "head":"0xb04009ddf4b0763f42778e7d5937e49bebf1e11b2d26c9dac6cefb5f84b6f8ea"
                },
                "snap":{}
            }
        }"#;

        let _: NodeInfo = serde_json::from_str(response).unwrap();
    }

    #[test]
    fn deserialize_node_info_post_merge() {
        // this response also has an enr
        let response = r#"{
            "id":"6e2fe698f3064cd99410926ce16734e35e3cc947d4354461d2594f2d2dd9f7b6",
            "name":"Geth/v1.10.19-stable/darwin-arm64/go1.18.3",
            "enode":"enode://d7dfaea49c7ef37701e668652bcf1bc63d3abb2ae97593374a949e175e4ff128730a2f35199f3462a56298b981dfc395a5abebd2d6f0284ffe5bdc3d8e258b86@127.0.0.1:30304?discport=0",
            "enr":"enr:-Jy4QIvS0dKBLjTTV_RojS8hjriwWsJNHRVyOh4Pk4aUXc5SZjKRVIOeYc7BqzEmbCjLdIY4Ln7x5ZPf-2SsBAc2_zqGAYSwY1zog2V0aMfGhNegsXuAgmlkgnY0gmlwhBiT_DiJc2VjcDI1NmsxoQLX366knH7zdwHmaGUrzxvGPTq7Kul1kzdKlJ4XXk_xKIRzbmFwwIN0Y3CCdmA",
            "ip":"127.0.0.1",
            "ports":{
                "discovery":0,
                "listener":30304
            },
            "listenAddr":"[::]:30304",
            "protocols":{
                "eth":{
                    "network":1337,
                    "difficulty":0,
                    "genesis":"0xb04009ddf4b0763f42778e7d5937e49bebf1e11b2d26c9dac6cefb5f84b6f8ea",
                    "config":{
                        "chainId":0,
                        "eip150Hash":"0x0000000000000000000000000000000000000000000000000000000000000000",
                        "terminalTotalDifficulty":58750000000000000000000,
                        "terminalTotalDifficultyPassed":true,
                        "ethash":{}
                    },
                    "head":"0xb04009ddf4b0763f42778e7d5937e49bebf1e11b2d26c9dac6cefb5f84b6f8ea"
                },
                "snap":{}
            }
        }"#;

        let _: NodeInfo = serde_json::from_str(response).unwrap();
    }

    #[test]
    fn deserialize_node_info_mainnet_full() {
        let actual_response = r#"{
            "id": "74477ca052fcf55ee9eafb369fafdb3e91ad7b64fbd7ae15a4985bfdc43696f2",
            "name": "Geth/v1.10.26-stable/darwin-arm64/go1.19.3",
            "enode": "enode://962184c6f2a19e064e2ddf0d5c5a788c8c5ed3a4909b7f75fb4dad967392ff542772bcc498cd7f15e13eecbde830265f379779c6da1f71fb8fe1a4734dfc0a1e@127.0.0.1:13337?discport=0",
            "enr": "enr:-J-4QFttJyL3f2-B2TQmBZNFxex99TSBv1YtB_8jqUbXWkf6LOREKQAPW2bIn8kJ8QvHbWxCQNFzTX6sehjbrz1ZkSuGAYSyQ0_rg2V0aMrJhPxk7ASDEYwwgmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQKWIYTG8qGeBk4t3w1cWniMjF7TpJCbf3X7Ta2Wc5L_VIRzbmFwwIN0Y3CCNBk",
            "ip": "127.0.0.1",
            "ports": {
                "discovery": 0,
                "listener": 13337
            },
            "listenAddr": "[::]:13337",
            "protocols": {
                "eth": {
                    "network": 1337,
                    "difficulty": 17179869184,
                    "genesis": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3",
                    "config": {
                        "chainId": 1,
                        "homesteadBlock": 1150000,
                        "daoForkBlock": 1920000,
                        "daoForkSupport": true,
                        "eip150Block": 2463000,
                        "eip150Hash": "0x2086799aeebeae135c246c65021c82b4e15a2c451340993aacfd2751886514f0",
                        "eip155Block": 2675000,
                        "eip158Block": 2675000,
                        "byzantiumBlock": 4370000,
                        "constantinopleBlock": 7280000,
                        "petersburgBlock": 7280000,
                        "istanbulBlock": 9069000,
                        "muirGlacierBlock": 9200000,
                        "berlinBlock": 12244000,
                        "londonBlock": 12965000,
                        "arrowGlacierBlock": 13773000,
                        "grayGlacierBlock": 15050000,
                        "terminalTotalDifficulty": 58750000000000000000000,
                        "terminalTotalDifficultyPassed": true,
                        "ethash": {}
                    },
                    "head": "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3"
                },
                "snap": {}
            }
        }"#;

        let _: NodeInfo = serde_json::from_str(actual_response).unwrap();
    }

    #[test]
    fn deserialize_peer_info_handshake() {
        let response = r#"{
            "enode": "enode://a997fde0023537ad01e536ebf2eeeb4b4b3d5286707586727b704f32e8e2b4959e08b6db5b27eb6b7e9f6efcbb53657f4e2bd16900aa77a89426dc3382c29ce0@[::1]:60948",
            "id": "df6f8bc331005962c2ef1f5236486a753bc6b2ddb5ef04370757999d1ca832d4",
            "name": "Geth/v1.10.26-stable-e5eb32ac/linux-amd64/go1.18.5",
            "caps": ["eth/66","eth/67","snap/1"],
            "network":{
                "localAddress":"[::1]:30304",
                "remoteAddress":"[::1]:60948",
                "inbound":true,
                "trusted":false,
                "static":false
            },
            "protocols":{
                "eth":"handshake",
                "snap":"handshake"
            }
        }"#;

        let info: PeerInfo = serde_json::from_str(response).unwrap();
        assert_eq!(info.protocols.eth, Some(EthPeerInfo::Handshake));
        assert_eq!(info.protocols.snap, Some(SnapPeerInfo::Handshake));
    }
}
