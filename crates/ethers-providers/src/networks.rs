//! Networks are used inside wallets and providers to ensure replay protection across networks,
//! as well as to allow functions to be called with ENS names instead of Addresses.
use ethers_types::{Address, H160, U64};

/// Trait for specifying network specific metadata, such as the Chain Id or the ENS
/// address.
pub trait Network {
    /// The network's Chain Id. If None, then EIP-155 is not used and as a result
    /// transactions **will not have replay protection**
    const CHAIN_ID: Option<U64>;

    /// The network's ENS address.
    const ENS_ADDRESS: Option<Address>;

    // TODO: Default providers? e.g. `mainnet.infura.io/XXX`?
}

/// Ethereum Mainnet, pre-specified ENS address and ChainID = 1 (for EIP-155)
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

/// Any other network, ChainID is not specified so **there is no replay protection when
/// using this network type**. ENS is also not specified, so any calls to the provider's
/// `lookup_address` and `resolve_name` _will fail_.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Any;

impl Network for Any {
    const CHAIN_ID: Option<U64> = None;
    const ENS_ADDRESS: Option<Address> = None;
}
