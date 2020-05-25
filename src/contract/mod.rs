use crate::types::Address;

mod abi;

pub struct Contract<ABI> {
    pub address: Address,
    pub abi: ABI,
}

impl<ABI> Contract<ABI> {
    pub fn new<A: Into<Address>>(address: A, abi: ABI) -> Self {
        Self {
            address: address.into(),
            abi,
        }
    }
}
