use ethers_core::types::Selector;

use serde::Deserialize;
use url::Url;

/// tokenURI(uint256)
pub const ERC721_SELECTOR: Selector = [0x63, 0x52, 0x21, 0x1e];

/// url(uint256)
pub const ERC1155_SELECTOR: Selector = [0x00, 0xfd, 0xd5, 0x8e];

const IPFS_GATEWAY: &str = "https://ipfs.io/ipfs/";

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
