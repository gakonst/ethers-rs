use ethers_core::{
    k256::{ecdsa::SigningKey, EncodedPoint as K256PublicKey},
    types::Address,
    utils::keccak256,
};

pub fn key_to_address(secret_key: &SigningKey) -> Address {
    // TODO: Can we do this in a better way?
    let uncompressed_pub_key = K256PublicKey::from(&secret_key.verify_key()).decompress();
    let public_key = uncompressed_pub_key.unwrap().to_bytes();
    debug_assert_eq!(public_key[0], 0x04);
    let hash = keccak256(&public_key[1..]);
    Address::from_slice(&hash[12..])
}
