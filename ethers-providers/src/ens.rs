//! [Ethereum Name Service](https://docs.ens.domains/) support
//! Adapted from https://github.com/hhatto/rust-ens/blob/master/src/lib.rs
use ethers_core::{
    types::{Address, NameOrAddress, Selector, TransactionRequest, H160, H256},
    utils::keccak256,
};

/// ENS registry address (`0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e`)
pub const ENS_ADDRESS: Address = H160([
    // cannot set type aliases as constructors
    0, 0, 0, 0, 0, 12, 46, 7, 78, 198, 154, 13, 251, 41, 151, 186, 108, 125, 46, 30,
]);

// Selectors
const ENS_REVERSE_REGISTRAR_DOMAIN: &str = "addr.reverse";

/// resolver(bytes32)
const RESOLVER: Selector = [1, 120, 184, 191];

/// addr(bytes32)
pub const ADDR_SELECTOR: Selector = [59, 59, 87, 222];

/// name(bytes32)
pub const NAME_SELECTOR: Selector = [105, 31, 52, 49];

/// Returns a transaction request for calling the `resolver` method on the ENS server
pub fn get_resolver<T: Into<Address>>(ens_address: T, name: &str) -> TransactionRequest {
    // keccak256('resolver(bytes32)')
    let data = [&RESOLVER[..], &namehash(name).0].concat();
    TransactionRequest {
        data: Some(data.into()),
        to: Some(NameOrAddress::Address(ens_address.into())),
        ..Default::default()
    }
}

/// Returns a transaction request for calling
pub fn resolve<T: Into<Address>>(
    resolver_address: T,
    selector: Selector,
    name: &str,
) -> TransactionRequest {
    let data = [&selector[..], &namehash(name).0].concat();
    TransactionRequest {
        data: Some(data.into()),
        to: Some(NameOrAddress::Address(resolver_address.into())),
        ..Default::default()
    }
}

/// Returns the reverse-registrar name of an address.
pub fn reverse_address(addr: Address) -> String {
    format!("{:?}.{}", addr, ENS_REVERSE_REGISTRAR_DOMAIN)[2..].to_string()
}

/// Returns the ENS namehash as specified in [EIP-137](https://eips.ethereum.org/EIPS/eip-137)
pub fn namehash(name: &str) -> H256 {
    if name.is_empty() {
        return H256::zero()
    }

    // iterate in reverse
    name.rsplit('.')
        .fold([0u8; 32], |node, label| keccak256(&[node, keccak256(label.as_bytes())].concat()))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_hex(hash: H256, val: &str) {
        let v = if let Some(stripped) = val.strip_prefix("0x") { stripped } else { val };

        assert_eq!(hash.0.to_vec(), hex::decode(v).unwrap());
    }

    #[test]
    fn test_namehash() {
        for (name, expected) in &[
            ("", "0000000000000000000000000000000000000000000000000000000000000000"),
            ("foo.eth", "de9b09fd7c5f901e23a3f19fecc54828e9c848539801e86591bd9801b019f84f"),
            ("eth", "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae"),
            ("alice.eth", "0x787192fc5378cc32aa956ddfdedbf26b24e8d78e40109add0eea2c1a012c3dec"),
        ] {
            assert_hex(namehash(name), expected);
        }
    }
}
