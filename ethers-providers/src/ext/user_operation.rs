use ethers_core::{types::{Address, Bytes, H256, U256, U64}, abi::{encode, encode_packed, Token}, utils::keccak256};
use serde::{Deserialize, Serialize};

/// UserOperation - a structure that describes a transaction to be sent on behalf of a user. To avoid confusion, it is not named “transaction”.
// Like a transaction, it contains “sender”, “to”, “calldata”, “maxFeePerGas”, “maxPriorityFee”, “signature”, “nonce”
/// See EIP-4337: Account Abstraction Using Alt Mempool.
#[derive(Debug, Default, Clone, PartialEq, Eq, Deserialize, Serialize)]
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

    ///Gas not calculated by the handleOps method, but added to the gas paid. Covers batch overhead.
    pub pre_verification_gas: U256,

    ///Same as EIP-1559 gas parameter.
    pub max_fee_per_gas: U256,

    ///Same as EIP-1559 gas parameter.
    pub max_priority_fee_per_gas: U256,

    ///If set, this field holds the paymaster address and paymaster-specific data.The paymaster will pay for the transaction instead of the sender.
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

    pub fn init_code(mut self, init_code: Bytes) -> Self {
        self.init_code = init_code;
        self
    }

    pub fn call_data(mut self, call_data: Bytes) -> Self {
        self.call_data = call_data;
        self
    }

    pub fn call_gas_limit(mut self, call_gas_limit: U256) -> Self {
        self.call_gas_limit = call_gas_limit;
        self
    }

    pub fn verification_gas_limit(mut self, verification_gas_limit: U256) -> Self {
        self.verification_gas_limit = verification_gas_limit;
        self
    }

    pub fn pre_verification_gas(mut self, pre_verification_gas: U256) -> Self {
        self.pre_verification_gas = pre_verification_gas;
        self
    }

    pub fn max_fee_per_gas(mut self, max_fee_per_gas: U256) -> Self {
        self.max_fee_per_gas = max_fee_per_gas;
        self
    }

    pub fn max_priority_fee_per_gas(mut self, max_priority_fee_per_gas: U256) -> Self {
        self.max_priority_fee_per_gas = max_priority_fee_per_gas;
        self
    }

    pub fn paymaster_and_data(mut self, paymaster_and_data: Bytes) -> Self {
        self.paymaster_and_data = paymaster_and_data;
        self
    }

    pub fn signature(mut self, signature: Bytes) -> Self {
        self.signature = signature;
        self
    }

    /// Pack the user operation data into bytes 
    pub fn pack(&self) -> Bytes {
        let encoded = encode(
            &[Token::Address(self.sender),
            Token::Uint(self.nonce),
            Token::Bytes(self.init_code.0.to_vec()),
            Token::Bytes(self.call_data.0.to_vec()),
            Token::Uint(self.call_gas_limit),
            Token::Uint(self.verification_gas_limit),
            Token::Uint(self.pre_verification_gas),
            Token::Uint(self.max_fee_per_gas),
            Token::Uint(self.max_priority_fee_per_gas),
            Token::Bytes(self.paymaster_and_data.0.to_vec()),
            Token::Bytes(self.signature.0.to_vec())
            ],
        );


        let encoded_bytes = Bytes::from(encoded);
        encoded_bytes
    }

    /// Pack the user operation data into bytes 
    pub fn pack_without_signature(&self) -> Bytes {
        let encoded = encode(
            &[Token::Address(self.sender),
            Token::Uint(self.nonce),
            Token::FixedBytes(keccak256(self.init_code.0.to_vec()).into()),
            Token::FixedBytes(keccak256(self.call_data.0.to_vec()).into()),
            Token::Uint(self.call_gas_limit),
            Token::Uint(self.verification_gas_limit),
            Token::Uint(self.pre_verification_gas),
            Token::Uint(self.max_fee_per_gas),
            Token::Uint(self.max_priority_fee_per_gas),
            Token::FixedBytes(keccak256(self.paymaster_and_data.0.to_vec()).into()),
            ],
        );

        let encoded_bytes = Bytes::from(encoded);
        encoded_bytes
    }

    /// calculate the hash of UserOperation
    pub fn cal_op_hash(&self) -> H256 {
        let op_hash = keccak256(self.pack_without_signature()).into();
        op_hash
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
                sender: "0x9c5754De1443984659E1b3a8d1931D83475ba29C".parse().unwrap(),
                nonce: 1.into(),
                init_code: "0x".parse().unwrap(),
                call_data: "0xb61d27f60000000000000000000000009c5754de1443984659e1b3a8d1931d83475ba29c00000000000000000000000000000000000000000000000000005af3107a400000000000000000000000000000000000000000000000000000000000000000600000000000000000000000000000000000000000000000000000000000000000".parse().unwrap(),
                call_gas_limit: 33_100.into(), 
                verification_gas_limit: 60_624.into(),
                pre_verification_gas: 44_056.into(),
                max_fee_per_gas: 1_695_000_030_u64.into(),
                max_priority_fee_per_gas: 1_695_000_000.into(),
                paymaster_and_data: Bytes::default(),
                signature: "0x37540ca4f91a9f08993ba4ebd4b7473902f69864c98951f9db8cb47b78764c1a13ad46894a96dc0cad68f9207e49b4dbb897f25f47f040cec2a636a8201c1cd71b".parse().unwrap(),
            },
        ];
        assert_eq!(uos[0].pack_without_signature(), "0x0000000000000000000000009c5754de1443984659e1b3a8d1931d83475ba29c0000000000000000000000000000000000000000000000000000000000000001c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470f7def7aeb687d6992b466243b713223689982cefca0f91a1f5c5f60adb532b93000000000000000000000000000000000000000000000000000000000000814c000000000000000000000000000000000000000000000000000000000000ecd0000000000000000000000000000000000000000000000000000000000000ac18000000000000000000000000000000000000000000000000000000006507a5de000000000000000000000000000000000000000000000000000000006507a5c0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470".parse::<Bytes>().unwrap());
        assert_eq!(uos[0].cal_op_hash(), "0x7c047664418d42d19b6d9e3aa1970f8c586924d33c2fd07558f33e75b1f4e586".parse::<H256>().unwrap());

    }


}