//! ERC4337 related utilities.
use ethers_core::{
    abi::{encode, Token},
    types::{Address, Bytes, Log, TransactionReceipt, H256, U256, U64},
    utils::keccak256,
};
use serde::{Deserialize, Serialize};

/// UserOperation - a structure that describes a transaction to be sent on behalf of a user. To
/// avoid confusion, it is not named “transaction”. Like a transaction, it contains “sender”, “to”,
/// “calldata”, “maxFeePerGas”, “maxPriorityFee”, “signature”, “nonce” See EIP-4337: Account
/// Abstraction Using Alt Mempool.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperation {
    /// The sender account of this request
    pub sender: Address,

    /// Unique value the sender uses to verify it is not a replay
    pub nonce: U256,

    /// If set, the account contract will be created by this constructor
    pub init_code: Bytes,

    /// The method call to execute on this account.
    pub call_data: Bytes,

    /// The gas limit passed to the callData method call.
    pub call_gas_limit: U256,

    ///Gas used for validateUserOp and validatePaymasterUserOp.
    pub verification_gas_limit: U256,

    ///Gas not calculated by the handleOps method, but added to the gas paid. Covers batch
    /// overhead.
    pub pre_verification_gas: U256,

    ///Same as EIP-1559 gas parameter.
    pub max_fee_per_gas: U256,

    ///Same as EIP-1559 gas parameter.
    pub max_priority_fee_per_gas: U256,

    ///If set, this field holds the paymaster address and paymaster-specific data.The paymaster
    /// will pay for the transaction instead of the sender.
    pub paymaster_and_data: Bytes,

    /// Sender-verified signature over the entire request, the EntryPoint address and the chain ID.
    pub signature: Bytes,
}

impl UserOperation {
    /// set sender
    pub fn sender(mut self, sender: Address) -> Self {
        self.sender = sender;
        self
    }

    /// set nonce
    pub fn nonce(mut self, nonce: U256) -> Self {
        self.nonce = nonce;
        self
    }

    /// set init_code
    pub fn init_code(mut self, init_code: Bytes) -> Self {
        self.init_code = init_code;
        self
    }

    /// set call data
    pub fn call_data(mut self, call_data: Bytes) -> Self {
        self.call_data = call_data;
        self
    }

    /// set call_gas_limit
    pub fn call_gas_limit(mut self, call_gas_limit: U256) -> Self {
        self.call_gas_limit = call_gas_limit;
        self
    }

    /// set verification_gas_limit
    pub fn verification_gas_limit(mut self, verification_gas_limit: U256) -> Self {
        self.verification_gas_limit = verification_gas_limit;
        self
    }

    /// set pre_verification_gas
    pub fn pre_verification_gas(mut self, pre_verification_gas: U256) -> Self {
        self.pre_verification_gas = pre_verification_gas;
        self
    }

    /// set max_fee_per_gas
    pub fn max_fee_per_gas(mut self, max_fee_per_gas: U256) -> Self {
        self.max_fee_per_gas = max_fee_per_gas;
        self
    }

    /// set max_priority_fee_per_gas
    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: U256) -> Self {
        self.max_priority_fee_per_gas = max_priority_fee_per_gas;
        self
    }

    /// set paymaster_and_data
    pub fn paymaster_and_data(mut self, paymaster_and_data: Bytes) -> Self {
        self.paymaster_and_data = paymaster_and_data;
        self
    }

    /// set signature
    pub fn signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }

    /// Pack the user operation data into bytes
    pub fn pack(&self) -> Bytes {
        let encoded = encode(&[
            Token::Address(self.sender),
            Token::Uint(self.nonce),
            Token::Bytes(self.init_code.0.to_vec()),
            Token::Bytes(self.call_data.0.to_vec()),
            Token::Uint(self.call_gas_limit),
            Token::Uint(self.verification_gas_limit),
            Token::Uint(self.pre_verification_gas),
            Token::Uint(self.max_fee_per_gas),
            Token::Uint(self.max_priority_fee_per_gas),
            Token::Bytes(self.paymaster_and_data.0.to_vec()),
            Token::Bytes(self.signature.0.to_vec()),
        ]);

        Bytes::from(encoded)
    }

    /// Pack the user operation data into bytes
    pub fn pack_without_signature(&self) -> Bytes {
        let encoded = encode(&[
            Token::Address(self.sender),
            Token::Uint(self.nonce),
            Token::FixedBytes(keccak256(&self.init_code.0).into()),
            Token::FixedBytes(keccak256(&self.call_data.0).into()),
            Token::Uint(self.call_gas_limit),
            Token::Uint(self.verification_gas_limit),
            Token::Uint(self.pre_verification_gas),
            Token::Uint(self.max_fee_per_gas),
            Token::Uint(self.max_priority_fee_per_gas),
            Token::FixedBytes(keccak256(&self.paymaster_and_data.0).into()),
        ]);

        Bytes::from(encoded)
    }

    /// calculate the hash of UserOperation
    pub fn cal_op_hash(&self) -> H256 {
        keccak256(self.pack_without_signature()).into()
    }

    /// calculate the hash of UserOperation
    pub fn cal_uo_hash(&self, entry_point: Address, chain_id: U256) -> H256 {
        let op_hash: H256 = keccak256(&self.pack_without_signature().0).into();
        H256::from_slice(
            keccak256(encode(&[
                Token::FixedBytes(op_hash.as_bytes().to_vec()),
                Token::Address(entry_point),
                Token::Uint(chain_id),
            ]))
            .as_slice(),
        )
    }
    /// Creates random user operation (for testing purposes)
    #[cfg(feature = "test-utils")]
    pub fn random() -> Self {
        UserOperation::default()
            .sender(Address::random())
            .verification_gas_limit(100_000.into())
            .pre_verification_gas(21_000.into())
            .max_priority_fee_per_gas(1_000_000_000.into())
    }
}

/// User operation hash
#[derive(
    Eq, Hash, PartialEq, Debug, Serialize, Deserialize, Clone, Copy, Default, PartialOrd, Ord,
)]
pub struct UserOperationHash(pub H256);

impl From<H256> for UserOperationHash {
    fn from(value: H256) -> Self {
        Self(value)
    }
}

impl From<UserOperationHash> for H256 {
    fn from(value: UserOperationHash) -> Self {
        value.0
    }
}

/// Gas estimations result for user operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationGasEstimation {
    ///gas overhead of this UserOperation
    pub pre_verification_gas: U256,

    ///actual gas used by the validation of this UserOperation
    pub verification_gas_limit: U256,

    ///value used by inner account execution
    pub call_gas_limit: U256,
}

/// Return a UserOperation based on a hash (userOpHash) returned by eth_sendUserOperation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationByHash {
    /// A structure that describes a transaction to be sent on behalf of a user
    pub user_operation: UserOperation,

    /// EntryPoint address
    pub entry_point: Address,

    /// Block hash of the block containing UserOperation
    pub block_hash: H256,

    /// Block number in which UserOperation is included
    pub block_number: U64,

    /// Transaction hash of the UserOperation
    pub transaction_hash: H256,
}

/// Return a UserOperation receipt based on a hash (userOpHash) returned by eth_sendUserOperation
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    /// The request hash
    pub user_op_hash: UserOperationHash,

    /// The sender account of this request
    pub sender: Address,

    /// Unique value the sender uses to verify it is not a replay
    pub nonce: U256,

    /// The paymaster used for this userOp (or empty)
    pub paymaster: Option<Address>,

    /// Actual amount paid (by account or paymaster) for this UserOperation
    pub actual_gas_cost: U256,

    /// total gas used by this UserOperation (including preVerification, creation, validation and
    /// execution)
    pub actual_gas_used: U256,

    /// Did this execution completed without revert
    pub success: bool,

    /// In case of revert, this is the revert reason
    pub reason: String,

    /// The logs generated by this UserOperation (not including logs of other UserOperations in the
    /// same bundle)
    pub logs: Vec<Log>,

    /// The TransactionReceipt object. Note that the returned TransactionReceipt is for the entire
    /// bundle, not only for this UserOperation.
    pub receipt: TransactionReceipt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_operation_pack() {
        let uos =  vec![
            UserOperation::default().verification_gas_limit(100_000.into()).pre_verification_gas(21_000.into()).max_priority_fee_per_gas(1_000_000_000.into()),
            UserOperation::default().sender("0x9c5754De1443984659E1b3a8d1931D83475ba29C".parse().unwrap()).call_gas_limit(200_000.into()).verification_gas_limit(100_000.into()).pre_verification_gas(21_000.into()).max_fee_per_gas(3_000_000_000_u64.into()).max_priority_fee_per_gas(1_000_000_000.into()).signature("0x7cb39607585dee8e297d0d7a669ad8c5e43975220b6773c10a138deadbc8ec864981de4b9b3c735288a217115fb33f8326a61ddabc60a534e3b5536515c70f931c".parse().unwrap()),
        ];
        assert_eq!(uos[0].pack(), "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001600000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000186a000000000000000000000000000000000000000000000000000000000000052080000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003b9aca0000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000".parse::<Bytes>().unwrap());
        assert_eq!(uos[1].pack(), "0x0000000000000000000000009c5754de1443984659e1b3a8d1931d83475ba29c0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000016000000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000030d4000000000000000000000000000000000000000000000000000000000000186a0000000000000000000000000000000000000000000000000000000000000520800000000000000000000000000000000000000000000000000000000b2d05e00000000000000000000000000000000000000000000000000000000003b9aca0000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000001c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000417cb39607585dee8e297d0d7a669ad8c5e43975220b6773c10a138deadbc8ec864981de4b9b3c735288a217115fb33f8326a61ddabc60a534e3b5536515c70f931c00000000000000000000000000000000000000000000000000000000000000".parse::<Bytes>().unwrap());
    }

    #[test]
    fn test_user_operation_hash() {
        let uos =  vec![
            UserOperation {
            sender: "0x921f125a92930cabb2969ad9323261d3a2a784e7".parse().unwrap(), 
            nonce: 0.into(),
            init_code: "0x9406cc6185a346906296840746125a0e449764545fbfb9cf00000000000000000000000043378ff8c70109ee4dbe85af34428ab0615ebd230000000000000000000000000000000000000000000000000000000000000000".parse::<Bytes>().unwrap(), 
            call_data: "0xb61d27f6000000000000000000000000a02bfd0ba5d182226627a933333ba92d1a60e234000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000".parse::<Bytes>().unwrap(), 
            call_gas_limit: 530_100.into(),
            verification_gas_limit: 500_624.into(),
            pre_verification_gas: 104_056.into(),
            max_fee_per_gas: 1_695_000_030.into(),
            max_priority_fee_per_gas: 1_695_000_000.into(),
            paymaster_and_data: Bytes::default(),
            signature: "0x5ae30c60c3ad36192f6efc38b3ac41d70d2c08fd8efc5a2f2457bfc17a4deea72fb6b40081dc8e05da85a5f05b977d15a9583fbe0d1766357d2553ad233ddd2f1c".parse::<Bytes>().unwrap()
        },
        ];
        let entry_point_address: Address =
            "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789".parse().unwrap();
        let chain_id = U256::from(5);
        assert_eq!(uos[0].pack_without_signature(), "0x000000000000000000000000921f125a92930cabb2969ad9323261d3a2a784e700000000000000000000000000000000000000000000000000000000000000008c7ec65f2478610babbba00a0ef4d343dfb054b4710761d5a21998c4accc5fe801e1ed1ec5f58d8c4d9a1c367d605d2be58bcf15aa2c09f4ac075deb572e164b00000000000000000000000000000000000000000000000000000000000816b4000000000000000000000000000000000000000000000000000000000007a3900000000000000000000000000000000000000000000000000000000000019678000000000000000000000000000000000000000000000000000000006507a5de000000000000000000000000000000000000000000000000000000006507a5c0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".parse::<Bytes>().unwrap());
        assert_eq!(
            uos[0].cal_op_hash(),
            "0x0f3fe7fc49990fb0faf26e30cf0cf56c9d74d90175a233cb294d0a3c76786143"
                .parse::<H256>()
                .unwrap()
        );
        assert_eq!(
            uos[0].cal_uo_hash(entry_point_address, chain_id),
            "0x7bca0c9a2ffbd23c25c7d5e1df0520142c0c39454cee778c3201eef6a8a27f06"
                .parse::<H256>()
                .unwrap()
        );
    }
}
