pub use multicallcontract::*;
#[allow(clippy::too_many_arguments, non_camel_case_types)]
mod multicallcontract {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    // Some macros like EthAbiType and EthAbiCodec expand into using "ethers_contract" which is not
    // defined here
    mod ethers_contract {
        pub use crate::*;
    }
    use crate::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers_core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers_providers::Middleware;
    #[doc = "Multicall was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    pub static MULTICALL_ABI: crate::Lazy<ethers_core::abi::Abi> = crate::Lazy::new(|| {
        ethers_core :: utils :: __serde_json :: from_str ("[\n    {\n        \"constant\": true,\n        \"inputs\": [],\n        \"name\": \"getCurrentBlockTimestamp\",\n        \"outputs\": [{ \"name\": \"timestamp\", \"type\": \"uint256\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": false,\n        \"inputs\": [\n            {\n                \"components\": [\n                    { \"name\": \"target\", \"type\": \"address\" },\n                    { \"name\": \"callData\", \"type\": \"bytes\" }\n                ],\n                \"name\": \"calls\",\n                \"type\": \"tuple[]\"\n            }\n        ],\n        \"name\": \"aggregate\",\n        \"outputs\": [\n            { \"name\": \"blockNumber\", \"type\": \"uint256\" },\n            { \"name\": \"returnData\", \"type\": \"bytes[]\" }\n        ],\n        \"payable\": false,\n        \"stateMutability\": \"nonpayable\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [],\n        \"name\": \"getLastBlockHash\",\n        \"outputs\": [{ \"name\": \"blockHash\", \"type\": \"bytes32\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [{ \"name\": \"addr\", \"type\": \"address\" }],\n        \"name\": \"getEthBalance\",\n        \"outputs\": [{ \"name\": \"balance\", \"type\": \"uint256\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [],\n        \"name\": \"getCurrentBlockDifficulty\",\n        \"outputs\": [{ \"name\": \"difficulty\", \"type\": \"uint256\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [],\n        \"name\": \"getCurrentBlockGasLimit\",\n        \"outputs\": [{ \"name\": \"gaslimit\", \"type\": \"uint256\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [],\n        \"name\": \"getCurrentBlockCoinbase\",\n        \"outputs\": [{ \"name\": \"coinbase\", \"type\": \"address\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    },\n    {\n        \"constant\": true,\n        \"inputs\": [{ \"name\": \"blockNumber\", \"type\": \"uint256\" }],\n        \"name\": \"getBlockHash\",\n        \"outputs\": [{ \"name\": \"blockHash\", \"type\": \"bytes32\" }],\n        \"payable\": false,\n        \"stateMutability\": \"view\",\n        \"type\": \"function\"\n    }\n]\n") . expect ("invalid abi")
    });
    pub struct Multicall<M>(crate::Contract<M>);
    impl<M> Clone for Multicall<M> {
        fn clone(&self) -> Self {
            Multicall(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for Multicall<M> {
        type Target = crate::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M: ethers_providers::Middleware> std::fmt::Debug for Multicall<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Multicall)).field(&self.address()).finish()
        }
    }
    impl<M: ethers_providers::Middleware> Multicall<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `crate`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            crate::Contract::new(address.into(), MULTICALL_ABI.clone(), client).into()
        }
        #[doc = "Calls the contract's `aggregate` (0x252dba42) function"]
        pub fn aggregate(
            &self,
            calls: ::std::vec::Vec<(ethers_core::types::Address, ethers_core::types::Bytes)>,
        ) -> crate::builders::ContractCall<
            M,
            (ethers_core::types::U256, ::std::vec::Vec<ethers_core::types::Bytes>),
        > {
            self.0
                .method_hash([37, 45, 186, 66], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBlockHash` (0xee82ac5e) function"]
        pub fn get_block_hash(
            &self,
            block_number: ethers_core::types::U256,
        ) -> crate::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([238, 130, 172, 94], block_number)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockCoinbase` (0xa8b0574e) function"]
        pub fn get_current_block_coinbase(
            &self,
        ) -> crate::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([168, 176, 87, 78], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockDifficulty` (0x72425d9d) function"]
        pub fn get_current_block_difficulty(
            &self,
        ) -> crate::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([114, 66, 93, 157], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockGasLimit` (0x86d516e8) function"]
        pub fn get_current_block_gas_limit(
            &self,
        ) -> crate::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([134, 213, 22, 232], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockTimestamp` (0x0f28c97d) function"]
        pub fn get_current_block_timestamp(
            &self,
        ) -> crate::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([15, 40, 201, 125], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getEthBalance` (0x4d2301cc) function"]
        pub fn get_eth_balance(
            &self,
            addr: ethers_core::types::Address,
        ) -> crate::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([77, 35, 1, 204], addr)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getLastBlockHash` (0x27e86d6e) function"]
        pub fn get_last_block_hash(&self) -> crate::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([39, 232, 109, 110], ())
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers_providers::Middleware> From<crate::Contract<M>> for Multicall<M> {
        fn from(contract: crate::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `[37, 45, 186, 66]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "aggregate", abi = "aggregate((address,bytes)[])")]
    pub struct AggregateCall {
        pub calls: ::std::vec::Vec<(ethers_core::types::Address, ethers_core::types::Bytes)>,
    }
    #[doc = "Container type for all input parameters for the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `[238, 130, 172, 94]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getBlockHash", abi = "getBlockHash(uint256)")]
    pub struct GetBlockHashCall {
        pub block_number: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `[168, 176, 87, 78]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getCurrentBlockCoinbase", abi = "getCurrentBlockCoinbase()")]
    pub struct GetCurrentBlockCoinbaseCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `[114, 66, 93, 157]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getCurrentBlockDifficulty", abi = "getCurrentBlockDifficulty()")]
    pub struct GetCurrentBlockDifficultyCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `[134, 213, 22, 232]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getCurrentBlockGasLimit", abi = "getCurrentBlockGasLimit()")]
    pub struct GetCurrentBlockGasLimitCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `[15, 40, 201, 125]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getCurrentBlockTimestamp", abi = "getCurrentBlockTimestamp()")]
    pub struct GetCurrentBlockTimestampCall;
    #[doc = "Container type for all input parameters for the `getEthBalance` function with signature `getEthBalance(address)` and selector `[77, 35, 1, 204]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getEthBalance", abi = "getEthBalance(address)")]
    pub struct GetEthBalanceCall {
        pub addr: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `[39, 232, 109, 110]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthCall, crate :: EthDisplay)]
    #[ethcall(name = "getLastBlockHash", abi = "getLastBlockHash()")]
    pub struct GetLastBlockHashCall;
    #[derive(Debug, Clone, PartialEq, Eq, crate :: EthAbiType)]
    pub enum MulticallCalls {
        Aggregate(AggregateCall),
        GetBlockHash(GetBlockHashCall),
        GetCurrentBlockCoinbase(GetCurrentBlockCoinbaseCall),
        GetCurrentBlockDifficulty(GetCurrentBlockDifficultyCall),
        GetCurrentBlockGasLimit(GetCurrentBlockGasLimitCall),
        GetCurrentBlockTimestamp(GetCurrentBlockTimestampCall),
        GetEthBalance(GetEthBalanceCall),
        GetLastBlockHash(GetLastBlockHashCall),
    }
    impl ethers_core::abi::AbiDecode for MulticallCalls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers_core::abi::AbiError> {
            if let Ok(decoded) =
                <AggregateCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::Aggregate(decoded))
            }
            if let Ok(decoded) =
                <GetBlockHashCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetBlockHash(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockCoinbaseCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetCurrentBlockCoinbase(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockDifficultyCall as ethers_core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(MulticallCalls::GetCurrentBlockDifficulty(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockGasLimitCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetCurrentBlockGasLimit(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockTimestampCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetCurrentBlockTimestamp(decoded))
            }
            if let Ok(decoded) =
                <GetEthBalanceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetEthBalance(decoded))
            }
            if let Ok(decoded) =
                <GetLastBlockHashCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(MulticallCalls::GetLastBlockHash(decoded))
            }
            Err(ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ethers_core::abi::AbiEncode for MulticallCalls {
        fn encode(self) -> Vec<u8> {
            match self {
                MulticallCalls::Aggregate(element) => element.encode(),
                MulticallCalls::GetBlockHash(element) => element.encode(),
                MulticallCalls::GetCurrentBlockCoinbase(element) => element.encode(),
                MulticallCalls::GetCurrentBlockDifficulty(element) => element.encode(),
                MulticallCalls::GetCurrentBlockGasLimit(element) => element.encode(),
                MulticallCalls::GetCurrentBlockTimestamp(element) => element.encode(),
                MulticallCalls::GetEthBalance(element) => element.encode(),
                MulticallCalls::GetLastBlockHash(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for MulticallCalls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                MulticallCalls::Aggregate(element) => element.fmt(f),
                MulticallCalls::GetBlockHash(element) => element.fmt(f),
                MulticallCalls::GetCurrentBlockCoinbase(element) => element.fmt(f),
                MulticallCalls::GetCurrentBlockDifficulty(element) => element.fmt(f),
                MulticallCalls::GetCurrentBlockGasLimit(element) => element.fmt(f),
                MulticallCalls::GetCurrentBlockTimestamp(element) => element.fmt(f),
                MulticallCalls::GetEthBalance(element) => element.fmt(f),
                MulticallCalls::GetLastBlockHash(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AggregateCall> for MulticallCalls {
        fn from(var: AggregateCall) -> Self {
            MulticallCalls::Aggregate(var)
        }
    }
    impl ::std::convert::From<GetBlockHashCall> for MulticallCalls {
        fn from(var: GetBlockHashCall) -> Self {
            MulticallCalls::GetBlockHash(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockCoinbaseCall> for MulticallCalls {
        fn from(var: GetCurrentBlockCoinbaseCall) -> Self {
            MulticallCalls::GetCurrentBlockCoinbase(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockDifficultyCall> for MulticallCalls {
        fn from(var: GetCurrentBlockDifficultyCall) -> Self {
            MulticallCalls::GetCurrentBlockDifficulty(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockGasLimitCall> for MulticallCalls {
        fn from(var: GetCurrentBlockGasLimitCall) -> Self {
            MulticallCalls::GetCurrentBlockGasLimit(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockTimestampCall> for MulticallCalls {
        fn from(var: GetCurrentBlockTimestampCall) -> Self {
            MulticallCalls::GetCurrentBlockTimestamp(var)
        }
    }
    impl ::std::convert::From<GetEthBalanceCall> for MulticallCalls {
        fn from(var: GetEthBalanceCall) -> Self {
            MulticallCalls::GetEthBalance(var)
        }
    }
    impl ::std::convert::From<GetLastBlockHashCall> for MulticallCalls {
        fn from(var: GetLastBlockHashCall) -> Self {
            MulticallCalls::GetLastBlockHash(var)
        }
    }
    #[doc = "Container type for all return fields from the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `[37, 45, 186, 66]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct AggregateReturn {
        pub block_number: ethers_core::types::U256,
        pub return_data: ::std::vec::Vec<ethers_core::types::Bytes>,
    }
    #[doc = "Container type for all return fields from the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `[238, 130, 172, 94]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetBlockHashReturn {
        pub block_hash: [u8; 32],
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `[168, 176, 87, 78]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetCurrentBlockCoinbaseReturn {
        pub coinbase: ethers_core::types::Address,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `[114, 66, 93, 157]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetCurrentBlockDifficultyReturn {
        pub difficulty: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `[134, 213, 22, 232]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetCurrentBlockGasLimitReturn {
        pub gaslimit: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `[15, 40, 201, 125]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetCurrentBlockTimestampReturn {
        pub timestamp: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getEthBalance` function with signature `getEthBalance(address)` and selector `[77, 35, 1, 204]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetEthBalanceReturn {
        pub balance: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `[39, 232, 109, 110]`"]
    #[derive(Clone, Debug, Default, Eq, PartialEq, crate :: EthAbiType, crate :: EthAbiCodec)]
    pub struct GetLastBlockHashReturn {
        pub block_hash: [u8; 32],
    }
}
