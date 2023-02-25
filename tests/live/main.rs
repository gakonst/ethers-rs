#![cfg(not(target_arch = "wasm32"))]

use ethers::{abi::Abi, prelude::*};

#[cfg(feature = "celo")]
mod celo;

/// Compiles the given contract and returns the ABI and Bytecode.
pub fn compile_contract(path: &str, name: &str) -> (Abi, Bytes) {
    let path = format!("../testdata/{path}");
    let compiled = Solc::default().compile_source(&path).unwrap();
    if compiled.has_error() {
        for err in compiled.errors {
            eprintln!("{err}");
        }
        panic!("Failed to compile");
    }
    let contract = compiled.get(&path, name).expect("could not find contract");
    let (abi, bin, _) = contract.into_parts_or_default();
    (abi, bin)
}
