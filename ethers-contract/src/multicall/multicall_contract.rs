pub use multicallcontract_mod::*;
mod multicallcontract_mod {
    #![allow(dead_code)]
    #![allow(unused_imports)]
    use crate::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers_core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers_providers::{JsonRpcClient, Middleware};
    #[doc = "MulticallContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static MULTICALLCONTRACT_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ( "[{\"inputs\":[{\"components\":[{\"internalType\":\"address\",\"name\":\"target\",\"type\":\"address\"},{\"internalType\":\"bytes\",\"name\":\"callData\",\"type\":\"bytes\"}],\"internalType\":\"struct MulticallContract.Call[]\",\"name\":\"calls\",\"type\":\"tuple[]\"}],\"name\":\"aggregate\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"internalType\":\"bytes[]\",\"name\":\"returnData\",\"type\":\"bytes[]\"}],\"stateMutability\":\"nonpayable\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"name\":\"getBlockHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getCurrentBlockCoinbase\",\"outputs\":[{\"internalType\":\"address\",\"name\":\"coinbase\",\"type\":\"address\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getCurrentBlockDifficulty\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"difficulty\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getCurrentBlockGasLimit\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"gaslimit\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getCurrentBlockTimestamp\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"timestamp\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[{\"internalType\":\"address\",\"name\":\"addr\",\"type\":\"address\"}],\"name\":\"getEthBalance\",\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\"}],\"stateMutability\":\"view\",\"type\":\"function\"},{\"inputs\":[],\"name\":\"getLastBlockHash\",\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\",\"type\":\"function\"}]" ) . expect ( "invalid abi" )
    });
    #[derive(Clone)]
    pub struct MulticallContract<M>(Contract<M>);
    impl<M> std::ops::Deref for MulticallContract<M> {
        type Target = Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: Middleware> std::fmt::Debug for MulticallContract<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(MulticallContract)).field(&self.address()).finish()
        }
    }
    impl<'a, M: Middleware> MulticallContract<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<Address>, C: Into<Arc<M>>>(address: T, client: C) -> Self {
            let contract =
                Contract::new(address.into(), MULTICALLCONTRACT_ABI.clone(), client.into());
            Self(contract)
        }
        #[doc = "Calls the contract's `aggregate` (0x252dba42) function"]
        pub fn aggregate(
            &self,
            calls: Vec<(Address, Bytes)>,
        ) -> ContractCall<M, (U256, Vec<Bytes>)> {
            self.0
                .method_hash([37, 45, 186, 66], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockDifficulty` (0x72425d9d) function"]
        pub fn get_current_block_difficulty(&self) -> ContractCall<M, U256> {
            self.0
                .method_hash([114, 66, 93, 157], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockGasLimit` (0x86d516e8) function"]
        pub fn get_current_block_gas_limit(&self) -> ContractCall<M, U256> {
            self.0
                .method_hash([134, 213, 22, 232], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockTimestamp` (0x0f28c97d) function"]
        pub fn get_current_block_timestamp(&self) -> ContractCall<M, U256> {
            self.0
                .method_hash([15, 40, 201, 125], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockCoinbase` (0xa8b0574e) function"]
        pub fn get_current_block_coinbase(&self) -> ContractCall<M, Address> {
            self.0
                .method_hash([168, 176, 87, 78], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBlockHash` (0xee82ac5e) function"]
        pub fn get_block_hash(&self, block_number: U256) -> ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([238, 130, 172, 94], block_number)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getEthBalance` (0x4d2301cc) function"]
        pub fn get_eth_balance(&self, addr: Address) -> ContractCall<M, U256> {
            self.0
                .method_hash([77, 35, 1, 204], addr)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getLastBlockHash` (0x27e86d6e) function"]
        pub fn get_last_block_hash(&self) -> ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([39, 232, 109, 110], ())
                .expect("method not found (this should never happen)")
        }
    }
}
