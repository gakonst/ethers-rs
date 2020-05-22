/// Parameters for instantiating a network
use ethereum_types::Address;

trait Network {
    const NAME: &'static str;
    const CHAIN_ID: u32;
    const ENS: Option<Address>;
}

#[derive(Clone, Debug)]
pub struct Mainnet;

impl Network for Mainnet {
    const NAME: &'static str = "mainnet";
    const CHAIN_ID: u32 = 1;
    // TODO: Replace with ENS address
    const ENS: Option<Address> = None;
}
