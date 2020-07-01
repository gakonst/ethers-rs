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
    use ethers_providers::JsonRpcClient;
    use ethers_signers::{Client, Signer};
    #[doc = "MulticallContract was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static MULTICALLCONTRACT_ABI: Lazy<Abi> = Lazy::new(|| {
        serde_json :: from_str ( "[{\n\t\"constant\": true,\n\t\"inputs\": [],\n\t\"name\": \"getCurrentBlockTimestamp\",\n\t\"outputs\": [{\n\t\t\"name\": \"timestamp\",\n\t\t\"type\": \"uint256\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [{\n\t\t\"components\": [{\n\t\t\t\"name\": \"target\",\n\t\t\t\"type\": \"address\"\n\t\t}, {\n\t\t\t\"name\": \"callData\",\n\t\t\t\"type\": \"bytes\"\n\t\t}],\n\t\t\"name\": \"calls\",\n\t\t\"type\": \"tuple[]\"\n\t}],\n\t\"name\": \"aggregate\",\n\t\"outputs\": [{\n\t\t\"name\": \"blockNumber\",\n\t\t\"type\": \"uint256\"\n\t}, {\n\t\t\"name\": \"returnData\",\n\t\t\"type\": \"bytes[]\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [],\n\t\"name\": \"getLastBlockHash\",\n\t\"outputs\": [{\n\t\t\"name\": \"blockHash\",\n\t\t\"type\": \"bytes32\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [{\n\t\t\"name\": \"addr\",\n\t\t\"type\": \"address\"\n\t}],\n\t\"name\": \"getEthBalance\",\n\t\"outputs\": [{\n\t\t\"name\": \"balance\",\n\t\t\"type\": \"uint256\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [],\n\t\"name\": \"getCurrentBlockDifficulty\",\n\t\"outputs\": [{\n\t\t\"name\": \"difficulty\",\n\t\t\"type\": \"uint256\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [],\n\t\"name\": \"getCurrentBlockGasLimit\",\n\t\"outputs\": [{\n\t\t\"name\": \"gaslimit\",\n\t\t\"type\": \"uint256\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [],\n\t\"name\": \"getCurrentBlockCoinbase\",\n\t\"outputs\": [{\n\t\t\"name\": \"coinbase\",\n\t\t\"type\": \"address\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}, {\n\t\"constant\": true,\n\t\"inputs\": [{\n\t\t\"name\": \"blockNumber\",\n\t\t\"type\": \"uint256\"\n\t}],\n\t\"name\": \"getBlockHash\",\n\t\"outputs\": [{\n\t\t\"name\": \"blockHash\",\n\t\t\"type\": \"bytes32\"\n\t}],\n\t\"payable\": false,\n\t\"stateMutability\": \"view\",\n\t\"type\": \"function\"\n}]\n" ) . expect ( "invalid abi" )
    });
    #[derive(Clone)]
    pub struct MulticallContract<P, S>(Contract<P, S>);
    impl<P, S> std::ops::Deref for MulticallContract<P, S> {
        type Target = Contract<P, S>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<P: JsonRpcClient, S: Signer> std::fmt::Debug for MulticallContract<P, S> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(MulticallContract))
                .field(&self.address())
                .finish()
        }
    }
    impl<'a, P: JsonRpcClient, S: Signer> MulticallContract<P, S> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `ethers::Contract`"]
        #[doc = r" object"]
        pub fn new<T: Into<Address>, C: Into<Arc<Client<P, S>>>>(address: T, client: C) -> Self {
            let contract =
                Contract::new(address.into(), MULTICALLCONTRACT_ABI.clone(), client.into());
            Self(contract)
        }
        #[doc = "Calls the contract's `getCurrentBlockTimestamp` (0x0f28c97d) function"]
        pub fn get_current_block_timestamp(&self) -> ContractCall<P, S, U256> {
            self.0
                .method_hash([15, 40, 201, 125], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getEthBalance` (0x4d2301cc) function"]
        pub fn get_eth_balance(&self, addr: Address) -> ContractCall<P, S, U256> {
            self.0
                .method_hash([77, 35, 1, 204], (addr,))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getLastBlockHash` (0x27e86d6e) function"]
        pub fn get_last_block_hash(&self) -> ContractCall<P, S, [u8; 32]> {
            self.0
                .method_hash([39, 232, 109, 110], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockGasLimit` (0x86d516e8) function"]
        pub fn get_current_block_gas_limit(&self) -> ContractCall<P, S, U256> {
            self.0
                .method_hash([134, 213, 22, 232], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBlockHash` (0xee82ac5e) function"]
        pub fn get_block_hash(&self, block_number: U256) -> ContractCall<P, S, [u8; 32]> {
            self.0
                .method_hash([238, 130, 172, 94], (block_number,))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockCoinbase` (0xa8b0574e) function"]
        pub fn get_current_block_coinbase(&self) -> ContractCall<P, S, Address> {
            self.0
                .method_hash([168, 176, 87, 78], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockDifficulty` (0x72425d9d) function"]
        pub fn get_current_block_difficulty(&self) -> ContractCall<P, S, U256> {
            self.0
                .method_hash([114, 66, 93, 157], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `aggregate` (0x252dba42) function"]
        pub fn aggregate(
            &self,
            calls: Vec<(Address, Vec<u8>)>,
        ) -> ContractCall<P, S, (U256, Vec<Vec<u8>>)> {
            self.0
                .method_hash([37, 45, 186, 66], calls)
                .expect("method not found (this should never happen)")
        }
    }
}
