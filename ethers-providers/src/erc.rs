//! ERC related utilities. Only supporting NFTs for now.
use ethers_core::types::{Address, Selector, U256};

use serde::Deserialize;
use std::str::FromStr;
use url::Url;

/// ownerOf(uint256 tokenId)
pub const ERC721_OWNER_SELECTOR: Selector = [0x63, 0x52, 0x21, 0x1e];

/// balanceOf(address owner, uint256 tokenId)
pub const ERC1155_BALANCE_SELECTOR: Selector = [0x00, 0xfd, 0xd5, 0x8e];

const IPFS_GATEWAY: &str = "https://ipfs.io/ipfs/";

/// An ERC 721 or 1155 token
pub struct ERCNFT {
    pub type_: ERCNFTType,
    pub contract: Address,
    pub id: [u8; 32],
}

impl FromStr for ERCNFT {
    type Err = String;
    fn from_str(input: &str) -> Result<ERCNFT, Self::Err> {
        let split: Vec<&str> =
            input.trim_start_matches("eip155:").trim_start_matches("1/").split(':').collect();
        let (token_type, inner_path) = if split.len() == 2 {
            (
                ERCNFTType::from_str(split[0])
                    .map_err(|_| "Unsupported ERC token type".to_string())?,
                split[1],
            )
        } else {
            return Err("Unsupported ERC link".to_string())
        };

        let token_split: Vec<&str> = inner_path.split('/').collect();
        let (contract_addr, token_id) = if token_split.len() == 2 {
            let token_id = U256::from_dec_str(token_split[1])
                .map_err(|e| format!("Unsupported token id type: {} {}", token_split[1], e))?;
            let mut token_id_bytes = [0x0; 32];
            token_id.to_big_endian(&mut token_id_bytes);
            (
                Address::from_str(token_split[0].trim_start_matches("0x"))
                    .map_err(|e| format!("Invalid contract address: {} {}", token_split[0], e))?,
                token_id_bytes,
            )
        } else {
            return Err("Unsupported ERC link path".to_string())
        };
        Ok(ERCNFT { id: token_id, type_: token_type, contract: contract_addr })
    }
}

/// Supported ERCs
#[derive(PartialEq)]
pub enum ERCNFTType {
    ERC721,
    ERC1155,
}

impl FromStr for ERCNFTType {
    type Err = ();
    fn from_str(input: &str) -> Result<ERCNFTType, Self::Err> {
        match input {
            "erc721" => Ok(ERCNFTType::ERC721),
            "erc1155" => Ok(ERCNFTType::ERC1155),
            _ => Err(()),
        }
    }
}

impl ERCNFTType {
    pub fn resolution_selector(&self) -> Selector {
        match self {
            // tokenURI(uint256)
            ERCNFTType::ERC721 => [0xc8, 0x7b, 0x56, 0xdd],
            // url(uint256)
            ERCNFTType::ERC1155 => [0x0e, 0x89, 0x34, 0x1c],
        }
    }
}

/// ERC-1155 and ERC-721 metadata document.
#[derive(Deserialize)]
pub struct Metadata {
    pub image: String,
}

/// Returns a HTTP url for an IPFS object.
pub fn http_link_ipfs(url: Url) -> Result<Url, String> {
    Url::parse(IPFS_GATEWAY)
        .unwrap()
        .join(url.to_string().trim_start_matches("ipfs://").trim_start_matches("ipfs/"))
        .map_err(|e| e.to_string())
}
