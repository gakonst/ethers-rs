use ethers_core::types::U256;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Debug)]
pub(crate) struct NonceManager {
    pub initialized: AtomicBool,
    pub nonce: AtomicU64,
}

impl NonceManager {
    /// Instantiates the nonce manager with a 0 nonce.
    pub fn new() -> Self {
        NonceManager {
            initialized: false.into(),
            nonce: 0.into(),
        }
    }

    /// Returns the next nonce to be used
    pub fn next(&self) -> U256 {
        let nonce = self.nonce.fetch_add(1, Ordering::SeqCst);
        nonce.into()
    }
}
