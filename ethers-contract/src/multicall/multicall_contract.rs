#[allow(clippy::too_many_arguments, non_camel_case_types)]
pub mod multicall_3 {
    #![allow(clippy::enum_variant_names)]
    #![allow(dead_code)]
    #![allow(clippy::type_complexity)]
    #![allow(unused_imports)]
    /// Some macros may expand into using `ethers_contract` instead of `crate`.
    mod ethers_contract {
        pub use crate::*;
    }

    // This is a hack to guarantee all ethers-derive macros can find the types.
    // See [`ethers_core::macros::determine_ethers_crates`]
    #[doc(hidden)]
    mod ethers {
        pub mod core {
            pub use ethers_core::*;
        }
        pub mod contract {
            pub use crate::*;
        }
        pub mod providers {
            pub use ethers_providers::*;
        }
    }

    use self::ethers_contract::{
        builders::{ContractCall, Event},
        Contract, Lazy,
    };
    use ethers_core::{
        abi::{Abi, Detokenize, InvalidOutputType, Token, Tokenizable},
        types::*,
    };
    use ethers_providers::Middleware;

    #[doc = "Multicall3 was auto-generated with ethers-rs Abigen. More information at: https://github.com/gakonst/ethers-rs"]
    use std::sync::Arc;
    # [rustfmt :: skip] const __ABI : & str = "[{\"type\":\"function\",\"name\":\"aggregate\",\"inputs\":[{\"internalType\":\"struct Multicall3.Call[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"internalType\":\"bytes[]\",\"name\":\"returnData\",\"type\":\"bytes[]\"}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"aggregate3\",\"inputs\":[{\"internalType\":\"struct Multicall3.Call3[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"struct Multicall3.Result[]\",\"name\":\"returnData\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"aggregate3Value\",\"inputs\":[{\"internalType\":\"struct Multicall3.Call3Value[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bool\"},{\"type\":\"uint256\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"struct Multicall3.Result[]\",\"name\":\"returnData\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"blockAndAggregate\",\"inputs\":[{\"internalType\":\"struct Multicall3.Call[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"},{\"internalType\":\"struct Multicall3.Result[]\",\"name\":\"returnData\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"getBasefee\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"basefee\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getBlockHash\",\"inputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getBlockNumber\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getChainId\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"chainid\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCurrentBlockCoinbase\",\"inputs\":[],\"outputs\":[{\"internalType\":\"address\",\"name\":\"coinbase\",\"type\":\"address\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCurrentBlockDifficulty\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"difficulty\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCurrentBlockGasLimit\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"gaslimit\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getCurrentBlockTimestamp\",\"inputs\":[],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"timestamp\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getEthBalance\",\"inputs\":[{\"internalType\":\"address\",\"name\":\"addr\",\"type\":\"address\"}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"balance\",\"type\":\"uint256\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"getLastBlockHash\",\"inputs\":[],\"outputs\":[{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"}],\"stateMutability\":\"view\"},{\"type\":\"function\",\"name\":\"tryAggregate\",\"inputs\":[{\"internalType\":\"bool\",\"name\":\"requireSuccess\",\"type\":\"bool\"},{\"internalType\":\"struct Multicall3.Call[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"struct Multicall3.Result[]\",\"name\":\"returnData\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"stateMutability\":\"payable\"},{\"type\":\"function\",\"name\":\"tryBlockAndAggregate\",\"inputs\":[{\"internalType\":\"bool\",\"name\":\"requireSuccess\",\"type\":\"bool\"},{\"internalType\":\"struct Multicall3.Call[]\",\"name\":\"calls\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"address\"},{\"type\":\"bytes\"}]}],\"outputs\":[{\"internalType\":\"uint256\",\"name\":\"blockNumber\",\"type\":\"uint256\"},{\"internalType\":\"bytes32\",\"name\":\"blockHash\",\"type\":\"bytes32\"},{\"internalType\":\"struct Multicall3.Result[]\",\"name\":\"returnData\",\"type\":\"tuple[]\",\"components\":[{\"type\":\"bool\"},{\"type\":\"bytes\"}]}],\"stateMutability\":\"payable\"}]" ;
    #[doc = r" The parsed JSON-ABI of the contract."]
    pub static MULTICALL3_ABI: ethers_contract::Lazy<ethers_core::abi::Abi> =
        ethers_contract::Lazy::new(|| {
            ethers_core::utils::__serde_json::from_str(__ABI).expect("invalid abi")
        });
    pub struct Multicall3<M>(ethers_contract::Contract<M>);
    impl<M> Clone for Multicall3<M> {
        fn clone(&self) -> Self {
            Multicall3(self.0.clone())
        }
    }
    impl<M> std::ops::Deref for Multicall3<M> {
        type Target = ethers_contract::Contract<M>;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl<M> std::fmt::Debug for Multicall3<M> {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.debug_tuple(stringify!(Multicall3)).field(&self.address()).finish()
        }
    }
    impl<M: ethers_providers::Middleware> Multicall3<M> {
        #[doc = r" Creates a new contract instance with the specified `ethers`"]
        #[doc = r" client at the given `Address`. The contract derefs to a `crate`"]
        #[doc = r" object"]
        pub fn new<T: Into<ethers_core::types::Address>>(
            address: T,
            client: ::std::sync::Arc<M>,
        ) -> Self {
            ethers_contract::Contract::new(address.into(), MULTICALL3_ABI.clone(), client).into()
        }
        #[doc = "Calls the contract's `aggregate` (0x252dba42) function"]
        pub fn aggregate(
            &self,
            calls: ::std::vec::Vec<Call>,
        ) -> ethers_contract::builders::ContractCall<
            M,
            (ethers_core::types::U256, ::std::vec::Vec<ethers_core::types::Bytes>),
        > {
            self.0
                .method_hash([37, 45, 186, 66], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `aggregate3` (0x82ad56cb) function"]
        pub fn aggregate_3(
            &self,
            calls: ::std::vec::Vec<Call3>,
        ) -> ethers_contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([130, 173, 86, 203], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `aggregate3Value` (0x174dea71) function"]
        pub fn aggregate_3_value(
            &self,
            calls: ::std::vec::Vec<Call3Value>,
        ) -> ethers_contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([23, 77, 234, 113], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `blockAndAggregate` (0xc3077fa9) function"]
        pub fn block_and_aggregate(
            &self,
            calls: ::std::vec::Vec<Call>,
        ) -> ethers_contract::builders::ContractCall<
            M,
            (ethers_core::types::U256, [u8; 32], ::std::vec::Vec<Result>),
        > {
            self.0
                .method_hash([195, 7, 127, 169], calls)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBasefee` (0x3e64a696) function"]
        pub fn get_basefee(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([62, 100, 166, 150], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBlockHash` (0xee82ac5e) function"]
        pub fn get_block_hash(
            &self,
            block_number: ethers_core::types::U256,
        ) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([238, 130, 172, 94], block_number)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getBlockNumber` (0x42cbb15c) function"]
        pub fn get_block_number(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([66, 203, 177, 92], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getChainId` (0x3408e470) function"]
        pub fn get_chain_id(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([52, 8, 228, 112], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockCoinbase` (0xa8b0574e) function"]
        pub fn get_current_block_coinbase(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::Address> {
            self.0
                .method_hash([168, 176, 87, 78], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockDifficulty` (0x72425d9d) function"]
        pub fn get_current_block_difficulty(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([114, 66, 93, 157], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockGasLimit` (0x86d516e8) function"]
        pub fn get_current_block_gas_limit(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([134, 213, 22, 232], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getCurrentBlockTimestamp` (0x0f28c97d) function"]
        pub fn get_current_block_timestamp(
            &self,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([15, 40, 201, 125], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getEthBalance` (0x4d2301cc) function"]
        pub fn get_eth_balance(
            &self,
            addr: ethers_core::types::Address,
        ) -> ethers_contract::builders::ContractCall<M, ethers_core::types::U256> {
            self.0
                .method_hash([77, 35, 1, 204], addr)
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `getLastBlockHash` (0x27e86d6e) function"]
        pub fn get_last_block_hash(&self) -> ethers_contract::builders::ContractCall<M, [u8; 32]> {
            self.0
                .method_hash([39, 232, 109, 110], ())
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tryAggregate` (0xbce38bd7) function"]
        pub fn try_aggregate(
            &self,
            require_success: bool,
            calls: ::std::vec::Vec<Call>,
        ) -> ethers_contract::builders::ContractCall<M, ::std::vec::Vec<Result>> {
            self.0
                .method_hash([188, 227, 139, 215], (require_success, calls))
                .expect("method not found (this should never happen)")
        }
        #[doc = "Calls the contract's `tryBlockAndAggregate` (0x399542e9) function"]
        pub fn try_block_and_aggregate(
            &self,
            require_success: bool,
            calls: ::std::vec::Vec<Call>,
        ) -> ethers_contract::builders::ContractCall<
            M,
            (ethers_core::types::U256, [u8; 32], ::std::vec::Vec<Result>),
        > {
            self.0
                .method_hash([57, 149, 66, 233], (require_success, calls))
                .expect("method not found (this should never happen)")
        }
    }
    impl<M: ethers_providers::Middleware> From<ethers_contract::Contract<M>> for Multicall3<M> {
        fn from(contract: ethers_contract::Contract<M>) -> Self {
            Self(contract)
        }
    }
    #[doc = "Container type for all input parameters for the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `[37, 45, 186, 66]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "aggregate", abi = "aggregate((address,bytes)[])")]
    pub struct AggregateCall {
        pub calls: ::std::vec::Vec<Call>,
    }
    #[doc = "Container type for all input parameters for the `aggregate3` function with signature `aggregate3((address,bool,bytes)[])` and selector `[130, 173, 86, 203]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "aggregate3", abi = "aggregate3((address,bool,bytes)[])")]
    pub struct Aggregate3Call {
        pub calls: ::std::vec::Vec<Call3>,
    }
    #[doc = "Container type for all input parameters for the `aggregate3Value` function with signature `aggregate3Value((address,bool,uint256,bytes)[])` and selector `[23, 77, 234, 113]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "aggregate3Value", abi = "aggregate3Value((address,bool,uint256,bytes)[])")]
    pub struct Aggregate3ValueCall {
        pub calls: ::std::vec::Vec<Call3Value>,
    }
    #[doc = "Container type for all input parameters for the `blockAndAggregate` function with signature `blockAndAggregate((address,bytes)[])` and selector `[195, 7, 127, 169]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "blockAndAggregate", abi = "blockAndAggregate((address,bytes)[])")]
    pub struct BlockAndAggregateCall {
        pub calls: ::std::vec::Vec<Call>,
    }
    #[doc = "Container type for all input parameters for the `getBasefee` function with signature `getBasefee()` and selector `[62, 100, 166, 150]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getBasefee", abi = "getBasefee()")]
    pub struct GetBasefeeCall;
    #[doc = "Container type for all input parameters for the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `[238, 130, 172, 94]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getBlockHash", abi = "getBlockHash(uint256)")]
    pub struct GetBlockHashCall {
        pub block_number: ethers_core::types::U256,
    }
    #[doc = "Container type for all input parameters for the `getBlockNumber` function with signature `getBlockNumber()` and selector `[66, 203, 177, 92]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getBlockNumber", abi = "getBlockNumber()")]
    pub struct GetBlockNumberCall;
    #[doc = "Container type for all input parameters for the `getChainId` function with signature `getChainId()` and selector `[52, 8, 228, 112]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getChainId", abi = "getChainId()")]
    pub struct GetChainIdCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `[168, 176, 87, 78]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getCurrentBlockCoinbase", abi = "getCurrentBlockCoinbase()")]
    pub struct GetCurrentBlockCoinbaseCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `[114, 66, 93, 157]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getCurrentBlockDifficulty", abi = "getCurrentBlockDifficulty()")]
    pub struct GetCurrentBlockDifficultyCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `[134, 213, 22, 232]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getCurrentBlockGasLimit", abi = "getCurrentBlockGasLimit()")]
    pub struct GetCurrentBlockGasLimitCall;
    #[doc = "Container type for all input parameters for the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `[15, 40, 201, 125]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getCurrentBlockTimestamp", abi = "getCurrentBlockTimestamp()")]
    pub struct GetCurrentBlockTimestampCall;
    #[doc = "Container type for all input parameters for the `getEthBalance` function with signature `getEthBalance(address)` and selector `[77, 35, 1, 204]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getEthBalance", abi = "getEthBalance(address)")]
    pub struct GetEthBalanceCall {
        pub addr: ethers_core::types::Address,
    }
    #[doc = "Container type for all input parameters for the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `[39, 232, 109, 110]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "getLastBlockHash", abi = "getLastBlockHash()")]
    pub struct GetLastBlockHashCall;
    #[doc = "Container type for all input parameters for the `tryAggregate` function with signature `tryAggregate(bool,(address,bytes)[])` and selector `[188, 227, 139, 215]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "tryAggregate", abi = "tryAggregate(bool,(address,bytes)[])")]
    pub struct TryAggregateCall {
        pub require_success: bool,
        pub calls: ::std::vec::Vec<Call>,
    }
    #[doc = "Container type for all input parameters for the `tryBlockAndAggregate` function with signature `tryBlockAndAggregate(bool,(address,bytes)[])` and selector `[57, 149, 66, 233]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthCall,
        ethers_contract :: EthDisplay,
    )]
    #[ethcall(name = "tryBlockAndAggregate", abi = "tryBlockAndAggregate(bool,(address,bytes)[])")]
    pub struct TryBlockAndAggregateCall {
        pub require_success: bool,
        pub calls: ::std::vec::Vec<Call>,
    }
    #[derive(Debug, Clone, PartialEq, Eq, ethers_contract :: EthAbiType)]
    pub enum Multicall3Calls {
        Aggregate(AggregateCall),
        Aggregate3(Aggregate3Call),
        Aggregate3Value(Aggregate3ValueCall),
        BlockAndAggregate(BlockAndAggregateCall),
        GetBasefee(GetBasefeeCall),
        GetBlockHash(GetBlockHashCall),
        GetBlockNumber(GetBlockNumberCall),
        GetChainId(GetChainIdCall),
        GetCurrentBlockCoinbase(GetCurrentBlockCoinbaseCall),
        GetCurrentBlockDifficulty(GetCurrentBlockDifficultyCall),
        GetCurrentBlockGasLimit(GetCurrentBlockGasLimitCall),
        GetCurrentBlockTimestamp(GetCurrentBlockTimestampCall),
        GetEthBalance(GetEthBalanceCall),
        GetLastBlockHash(GetLastBlockHashCall),
        TryAggregate(TryAggregateCall),
        TryBlockAndAggregate(TryBlockAndAggregateCall),
    }
    impl ethers_core::abi::AbiDecode for Multicall3Calls {
        fn decode(
            data: impl AsRef<[u8]>,
        ) -> ::std::result::Result<Self, ethers_core::abi::AbiError> {
            if let Ok(decoded) =
                <AggregateCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::Aggregate(decoded))
            }
            if let Ok(decoded) =
                <Aggregate3Call as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::Aggregate3(decoded))
            }
            if let Ok(decoded) =
                <Aggregate3ValueCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::Aggregate3Value(decoded))
            }
            if let Ok(decoded) =
                <BlockAndAggregateCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::BlockAndAggregate(decoded))
            }
            if let Ok(decoded) =
                <GetBasefeeCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetBasefee(decoded))
            }
            if let Ok(decoded) =
                <GetBlockHashCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetBlockHash(decoded))
            }
            if let Ok(decoded) =
                <GetBlockNumberCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetBlockNumber(decoded))
            }
            if let Ok(decoded) =
                <GetChainIdCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetChainId(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockCoinbaseCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetCurrentBlockCoinbase(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockDifficultyCall as ethers_core::abi::AbiDecode>::decode(
                    data.as_ref(),
                )
            {
                return Ok(Multicall3Calls::GetCurrentBlockDifficulty(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockGasLimitCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetCurrentBlockGasLimit(decoded))
            }
            if let Ok(decoded) =
                <GetCurrentBlockTimestampCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetCurrentBlockTimestamp(decoded))
            }
            if let Ok(decoded) =
                <GetEthBalanceCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetEthBalance(decoded))
            }
            if let Ok(decoded) =
                <GetLastBlockHashCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::GetLastBlockHash(decoded))
            }
            if let Ok(decoded) =
                <TryAggregateCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::TryAggregate(decoded))
            }
            if let Ok(decoded) =
                <TryBlockAndAggregateCall as ethers_core::abi::AbiDecode>::decode(data.as_ref())
            {
                return Ok(Multicall3Calls::TryBlockAndAggregate(decoded))
            }
            Err(ethers_core::abi::Error::InvalidData.into())
        }
    }
    impl ethers_core::abi::AbiEncode for Multicall3Calls {
        fn encode(self) -> Vec<u8> {
            match self {
                Multicall3Calls::Aggregate(element) => element.encode(),
                Multicall3Calls::Aggregate3(element) => element.encode(),
                Multicall3Calls::Aggregate3Value(element) => element.encode(),
                Multicall3Calls::BlockAndAggregate(element) => element.encode(),
                Multicall3Calls::GetBasefee(element) => element.encode(),
                Multicall3Calls::GetBlockHash(element) => element.encode(),
                Multicall3Calls::GetBlockNumber(element) => element.encode(),
                Multicall3Calls::GetChainId(element) => element.encode(),
                Multicall3Calls::GetCurrentBlockCoinbase(element) => element.encode(),
                Multicall3Calls::GetCurrentBlockDifficulty(element) => element.encode(),
                Multicall3Calls::GetCurrentBlockGasLimit(element) => element.encode(),
                Multicall3Calls::GetCurrentBlockTimestamp(element) => element.encode(),
                Multicall3Calls::GetEthBalance(element) => element.encode(),
                Multicall3Calls::GetLastBlockHash(element) => element.encode(),
                Multicall3Calls::TryAggregate(element) => element.encode(),
                Multicall3Calls::TryBlockAndAggregate(element) => element.encode(),
            }
        }
    }
    impl ::std::fmt::Display for Multicall3Calls {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Multicall3Calls::Aggregate(element) => element.fmt(f),
                Multicall3Calls::Aggregate3(element) => element.fmt(f),
                Multicall3Calls::Aggregate3Value(element) => element.fmt(f),
                Multicall3Calls::BlockAndAggregate(element) => element.fmt(f),
                Multicall3Calls::GetBasefee(element) => element.fmt(f),
                Multicall3Calls::GetBlockHash(element) => element.fmt(f),
                Multicall3Calls::GetBlockNumber(element) => element.fmt(f),
                Multicall3Calls::GetChainId(element) => element.fmt(f),
                Multicall3Calls::GetCurrentBlockCoinbase(element) => element.fmt(f),
                Multicall3Calls::GetCurrentBlockDifficulty(element) => element.fmt(f),
                Multicall3Calls::GetCurrentBlockGasLimit(element) => element.fmt(f),
                Multicall3Calls::GetCurrentBlockTimestamp(element) => element.fmt(f),
                Multicall3Calls::GetEthBalance(element) => element.fmt(f),
                Multicall3Calls::GetLastBlockHash(element) => element.fmt(f),
                Multicall3Calls::TryAggregate(element) => element.fmt(f),
                Multicall3Calls::TryBlockAndAggregate(element) => element.fmt(f),
            }
        }
    }
    impl ::std::convert::From<AggregateCall> for Multicall3Calls {
        fn from(var: AggregateCall) -> Self {
            Multicall3Calls::Aggregate(var)
        }
    }
    impl ::std::convert::From<Aggregate3Call> for Multicall3Calls {
        fn from(var: Aggregate3Call) -> Self {
            Multicall3Calls::Aggregate3(var)
        }
    }
    impl ::std::convert::From<Aggregate3ValueCall> for Multicall3Calls {
        fn from(var: Aggregate3ValueCall) -> Self {
            Multicall3Calls::Aggregate3Value(var)
        }
    }
    impl ::std::convert::From<BlockAndAggregateCall> for Multicall3Calls {
        fn from(var: BlockAndAggregateCall) -> Self {
            Multicall3Calls::BlockAndAggregate(var)
        }
    }
    impl ::std::convert::From<GetBasefeeCall> for Multicall3Calls {
        fn from(var: GetBasefeeCall) -> Self {
            Multicall3Calls::GetBasefee(var)
        }
    }
    impl ::std::convert::From<GetBlockHashCall> for Multicall3Calls {
        fn from(var: GetBlockHashCall) -> Self {
            Multicall3Calls::GetBlockHash(var)
        }
    }
    impl ::std::convert::From<GetBlockNumberCall> for Multicall3Calls {
        fn from(var: GetBlockNumberCall) -> Self {
            Multicall3Calls::GetBlockNumber(var)
        }
    }
    impl ::std::convert::From<GetChainIdCall> for Multicall3Calls {
        fn from(var: GetChainIdCall) -> Self {
            Multicall3Calls::GetChainId(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockCoinbaseCall> for Multicall3Calls {
        fn from(var: GetCurrentBlockCoinbaseCall) -> Self {
            Multicall3Calls::GetCurrentBlockCoinbase(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockDifficultyCall> for Multicall3Calls {
        fn from(var: GetCurrentBlockDifficultyCall) -> Self {
            Multicall3Calls::GetCurrentBlockDifficulty(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockGasLimitCall> for Multicall3Calls {
        fn from(var: GetCurrentBlockGasLimitCall) -> Self {
            Multicall3Calls::GetCurrentBlockGasLimit(var)
        }
    }
    impl ::std::convert::From<GetCurrentBlockTimestampCall> for Multicall3Calls {
        fn from(var: GetCurrentBlockTimestampCall) -> Self {
            Multicall3Calls::GetCurrentBlockTimestamp(var)
        }
    }
    impl ::std::convert::From<GetEthBalanceCall> for Multicall3Calls {
        fn from(var: GetEthBalanceCall) -> Self {
            Multicall3Calls::GetEthBalance(var)
        }
    }
    impl ::std::convert::From<GetLastBlockHashCall> for Multicall3Calls {
        fn from(var: GetLastBlockHashCall) -> Self {
            Multicall3Calls::GetLastBlockHash(var)
        }
    }
    impl ::std::convert::From<TryAggregateCall> for Multicall3Calls {
        fn from(var: TryAggregateCall) -> Self {
            Multicall3Calls::TryAggregate(var)
        }
    }
    impl ::std::convert::From<TryBlockAndAggregateCall> for Multicall3Calls {
        fn from(var: TryBlockAndAggregateCall) -> Self {
            Multicall3Calls::TryBlockAndAggregate(var)
        }
    }
    #[doc = "Container type for all return fields from the `aggregate` function with signature `aggregate((address,bytes)[])` and selector `[37, 45, 186, 66]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct AggregateReturn {
        pub block_number: ethers_core::types::U256,
        pub return_data: ::std::vec::Vec<ethers_core::types::Bytes>,
    }
    #[doc = "Container type for all return fields from the `aggregate3` function with signature `aggregate3((address,bool,bytes)[])` and selector `[130, 173, 86, 203]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Aggregate3Return {
        pub return_data: ::std::vec::Vec<Result>,
    }
    #[doc = "Container type for all return fields from the `aggregate3Value` function with signature `aggregate3Value((address,bool,uint256,bytes)[])` and selector `[23, 77, 234, 113]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Aggregate3ValueReturn {
        pub return_data: ::std::vec::Vec<Result>,
    }
    #[doc = "Container type for all return fields from the `blockAndAggregate` function with signature `blockAndAggregate((address,bytes)[])` and selector `[195, 7, 127, 169]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct BlockAndAggregateReturn {
        pub block_number: ethers_core::types::U256,
        pub block_hash: [u8; 32],
        pub return_data: ::std::vec::Vec<Result>,
    }
    #[doc = "Container type for all return fields from the `getBasefee` function with signature `getBasefee()` and selector `[62, 100, 166, 150]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetBasefeeReturn {
        pub basefee: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getBlockHash` function with signature `getBlockHash(uint256)` and selector `[238, 130, 172, 94]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetBlockHashReturn {
        pub block_hash: [u8; 32],
    }
    #[doc = "Container type for all return fields from the `getBlockNumber` function with signature `getBlockNumber()` and selector `[66, 203, 177, 92]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetBlockNumberReturn {
        pub block_number: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getChainId` function with signature `getChainId()` and selector `[52, 8, 228, 112]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetChainIdReturn {
        pub chainid: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockCoinbase` function with signature `getCurrentBlockCoinbase()` and selector `[168, 176, 87, 78]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetCurrentBlockCoinbaseReturn {
        pub coinbase: ethers_core::types::Address,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockDifficulty` function with signature `getCurrentBlockDifficulty()` and selector `[114, 66, 93, 157]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetCurrentBlockDifficultyReturn {
        pub difficulty: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockGasLimit` function with signature `getCurrentBlockGasLimit()` and selector `[134, 213, 22, 232]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetCurrentBlockGasLimitReturn {
        pub gaslimit: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getCurrentBlockTimestamp` function with signature `getCurrentBlockTimestamp()` and selector `[15, 40, 201, 125]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetCurrentBlockTimestampReturn {
        pub timestamp: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getEthBalance` function with signature `getEthBalance(address)` and selector `[77, 35, 1, 204]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetEthBalanceReturn {
        pub balance: ethers_core::types::U256,
    }
    #[doc = "Container type for all return fields from the `getLastBlockHash` function with signature `getLastBlockHash()` and selector `[39, 232, 109, 110]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct GetLastBlockHashReturn {
        pub block_hash: [u8; 32],
    }
    #[doc = "Container type for all return fields from the `tryAggregate` function with signature `tryAggregate(bool,(address,bytes)[])` and selector `[188, 227, 139, 215]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct TryAggregateReturn {
        pub return_data: ::std::vec::Vec<Result>,
    }
    #[doc = "Container type for all return fields from the `tryBlockAndAggregate` function with signature `tryBlockAndAggregate(bool,(address,bytes)[])` and selector `[57, 149, 66, 233]`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct TryBlockAndAggregateReturn {
        pub block_number: ethers_core::types::U256,
        pub block_hash: [u8; 32],
        pub return_data: ::std::vec::Vec<Result>,
    }
    #[doc = "`Call(address,bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Call {
        pub target: ethers_core::types::Address,
        pub call_data: ethers_core::types::Bytes,
    }
    #[doc = "`Call3(address,bool,bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Call3 {
        pub target: ethers_core::types::Address,
        pub allow_failure: bool,
        pub call_data: ethers_core::types::Bytes,
    }
    #[doc = "`Call3Value(address,bool,uint256,bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Call3Value {
        pub target: ethers_core::types::Address,
        pub allow_failure: bool,
        pub value: ethers_core::types::U256,
        pub call_data: ethers_core::types::Bytes,
    }
    #[doc = "`Result(bool,bytes)`"]
    #[derive(
        Clone,
        Debug,
        Default,
        Eq,
        PartialEq,
        ethers_contract :: EthAbiType,
        ethers_contract :: EthAbiCodec,
    )]
    pub struct Result {
        pub success: bool,
        pub return_data: ethers_core::types::Bytes,
    }
}
