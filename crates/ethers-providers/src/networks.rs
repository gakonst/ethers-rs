//! Networks are used inside wallets to ensure type-safety across networks. That way
//! a transaction that is designed to work with testnet does not accidentally work
//! with mainnet because the URL was changed.

use ethers_types::{Address, H160, U64};

pub trait Network {
    const CHAIN_ID: Option<U64>;
    const ENS_ADDRESS: Option<Address>;

    // TODO: Default providers? e.g. `mainnet.infura.io/XXX`?
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Mainnet;

impl Network for Mainnet {
    const CHAIN_ID: Option<U64> = Some(U64([1]));

    // 0x00000000000C2E074eC69A0dFb2997BA6C7d2e1e
    const ENS_ADDRESS: Option<Address> = Some(H160([
        // cannot set type aliases as constructors
        0, 0, 0, 0, 0, 12, 46, 7, 78, 198, 154, 13, 251, 41, 151, 186, 108, 125, 46, 30,
    ]));
}

/// No EIP155
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Any;

impl Network for Any {
    const CHAIN_ID: Option<U64> = None;
    const ENS_ADDRESS: Option<Address> = None;
}
