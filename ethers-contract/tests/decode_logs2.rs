use ethers_contract::{abigen, EthEvent};
// use ethers_core::abi::Tokenizable;
//
abigen!(DsProxyFactory,
    "ethers-middleware/contracts/DsProxyFactory.json",
    methods {
        build(address) as build_with_owner;
    }
);
