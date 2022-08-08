//! Types for the Parity Transaction-Trace Filtering API
use crate::types::{Address, BlockNumber, Bytes, H160, H256, U256};
use serde::{Deserialize, Serialize};

/// Trace filter
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraceFilter {
    /// From block
    #[serde(rename = "fromBlock", skip_serializing_if = "Option::is_none")]
    from_block: Option<BlockNumber>,
    /// To block
    #[serde(rename = "toBlock", skip_serializing_if = "Option::is_none")]
    to_block: Option<BlockNumber>,
    /// From address
    #[serde(rename = "fromAddress", skip_serializing_if = "Option::is_none")]
    from_address: Option<Vec<Address>>,
    /// To address
    #[serde(rename = "toAddress", skip_serializing_if = "Option::is_none")]
    to_address: Option<Vec<Address>>,
    /// Output offset
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<usize>,
    /// Output amount
    #[serde(skip_serializing_if = "Option::is_none")]
    count: Option<usize>,
}

impl TraceFilter {
    /// Sets From block
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn from_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.from_block = Some(block.into());
        self
    }

    /// Sets to block
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_block<T: Into<BlockNumber>>(mut self, block: T) -> Self {
        self.to_block = Some(block.into());
        self
    }

    /// Sets to address
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn to_address(mut self, address: Vec<H160>) -> Self {
        self.to_address = Some(address);
        self
    }

    /// Sets from address
    #[allow(clippy::wrong_self_convention)]
    #[must_use]
    pub fn from_address(mut self, address: Vec<H160>) -> Self {
        self.from_address = Some(address);
        self
    }

    /// Sets after offset
    #[must_use]
    pub fn after(mut self, after: usize) -> Self {
        self.after = Some(after);
        self
    }

    /// Sets amount of traces to display
    #[must_use]
    pub fn count(mut self, count: usize) -> Self {
        self.count = Some(count);
        self
    }
}

// `LocalizedTrace` in Parity
/// Trace-Filtering API trace type
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub struct Trace {
    /// Action
    pub action: Action,
    /// Result
    pub result: Option<Res>,
    /// Trace address
    #[serde(rename = "traceAddress")]
    pub trace_address: Vec<usize>,
    /// Subtraces
    pub subtraces: usize,
    /// Transaction position
    #[serde(rename = "transactionPosition")]
    pub transaction_position: Option<usize>,
    /// Transaction hash
    #[serde(rename = "transactionHash")]
    pub transaction_hash: Option<H256>,
    /// Block Number
    #[serde(rename = "blockNumber")]
    pub block_number: u64,
    /// Block Hash
    #[serde(rename = "blockHash")]
    pub block_hash: H256,
    /// Action Type
    #[serde(rename = "type")]
    pub action_type: ActionType,
    /// Error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Response
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Res {
    /// Call
    Call(CallResult),
    /// Create
    Create(CreateResult),
    /// None
    None,
}

impl Default for Res {
    fn default() -> Res {
        Res::None
    }
}

/// Action
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(untagged, rename_all = "lowercase")]
pub enum Action {
    /// Call
    Call(Call),
    /// Create
    Create(Create),
    /// Suicide
    Suicide(Suicide),
    /// Reward
    Reward(Reward),
}

/// An external action type.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    /// Contract call.
    Call,
    /// Contract creation.
    Create,
    /// Contract suicide.
    Suicide,
    /// A block reward.
    Reward,
}

/// Call Result
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CallResult {
    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Output bytes
    pub output: Bytes,
}

/// Create Result
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct CreateResult {
    /// Gas used
    #[serde(rename = "gasUsed")]
    pub gas_used: U256,
    /// Code
    pub code: Bytes,
    /// Assigned address
    pub address: Address,
}

/// Call response
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Call {
    /// Sender
    pub from: Address,
    /// Recipient
    pub to: Address,
    /// Transferred Value
    pub value: U256,
    /// Gas
    pub gas: U256,
    /// Input data
    pub input: Bytes,
    /// The type of the call.
    #[serde(rename = "callType")]
    pub call_type: CallType,
}

/// Call type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum CallType {
    /// None
    #[serde(rename = "none")]
    None,
    /// Call
    #[serde(rename = "call")]
    Call,
    /// Call code
    #[serde(rename = "callcode")]
    CallCode,
    /// Delegate call
    #[serde(rename = "delegatecall")]
    DelegateCall,
    /// Static call
    #[serde(rename = "staticcall")]
    StaticCall,
}

impl Default for CallType {
    fn default() -> CallType {
        CallType::None
    }
}

/// Create response
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Create {
    /// Sender
    pub from: Address,
    /// Value
    pub value: U256,
    /// Gas
    pub gas: U256,
    /// Initialization code
    pub init: Bytes,
}

/// Suicide
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct Suicide {
    /// Address.
    pub address: Address,
    /// Refund address.
    #[serde(rename = "refundAddress")]
    pub refund_address: Address,
    /// Balance.
    pub balance: U256,
}

/// Reward action
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Reward {
    /// Author's address.
    pub author: Address,
    /// Reward amount.
    pub value: U256,
    /// Reward type.
    #[serde(rename = "rewardType")]
    pub reward_type: RewardType,
}

/// Reward type.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum RewardType {
    /// Block
    #[serde(rename = "block")]
    Block,
    /// Uncle
    #[serde(rename = "uncle")]
    Uncle,
    /// EmptyStep (AuthorityRound)
    #[serde(rename = "emptyStep")]
    EmptyStep,
    /// External (attributed as part of an external protocol)
    #[serde(rename = "external")]
    External,
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_TRACE_CALL: &str = r#"{
        "action": {
            "callType": "call",
            "from": "0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            "gas": "0x63ab9",
            "input": "0xb9f256cd000000000000000000000000fb6916095ca1df60bb79ce92ce3ea74c37c5d3590000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000e85468697320697320746865206f6666696369616c20457468657265756d20466f756e646174696f6e20546970204a61722e20466f722065766572792061626f76652061206365727461696e2076616c756520646f6e6174696f6e207765276c6c2063726561746520616e642073656e6420746f20796f752061206272616e64206e657720556e69636f726e20546f6b656e2028f09fa684292e20436865636b2074686520756e69636f726e2070726963652062656c6f77202831206574686572203d20313030302066696e6e6579292e205468616e6b7320666f722074686520737570706f72742100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "to": "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359",
            "value": "0x0"
        },
        "blockHash": "0x6474a53a9ebf72d306a1406ec12ded12e210b6c3141b4373bfb3a3cea987dfb8",
        "blockNumber": 988775,
        "result": {
            "gasUsed": "0x4b419",
            "output": "0x0000000000000000000000000000000000000000000000000000000000000000"
        },
        "subtraces": 1,
        "traceAddress": [],
        "transactionHash": "0x342c284238149db221f9d87db87f90ffad7ac0aac57c0c480142f4c21b63f652",
        "transactionPosition": 1,
        "type": "call"
    }"#;

    const EXAMPLE_TRACE_CREATE: &str = r#"{
        "action": {
            "from": "0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            "gas": "0x63ab9",
            "init": "0xb9f256cd000000000000000000000000fb6916095ca1df60bb79ce92ce3ea74c37c5d3590000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000001a000000000000000000000000000000000000000000000000000000000000000e85468697320697320746865206f6666696369616c20457468657265756d20466f756e646174696f6e20546970204a61722e20466f722065766572792061626f76652061206365727461696e2076616c756520646f6e6174696f6e207765276c6c2063726561746520616e642073656e6420746f20796f752061206272616e64206e657720556e69636f726e20546f6b656e2028f09fa684292e20436865636b2074686520756e69636f726e2070726963652062656c6f77202831206574686572203d20313030302066696e6e6579292e205468616e6b7320666f722074686520737570706f72742100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            "value": "0x0"
        },
        "blockHash": "0x6474a53a9ebf72d306a1406ec12ded12e210b6c3141b4373bfb3a3cea987dfb8",
        "blockNumber": 988775,
        "result": {
            "gasUsed": "0x4b419",
            "output": "0x0000000000000000000000000000000000000000000000000000000000000000"
        },
        "subtraces": 1,
        "traceAddress": [],
        "transactionHash": "0x342c284238149db221f9d87db87f90ffad7ac0aac57c0c480142f4c21b63f652",
        "transactionPosition": 1,
        "type": "create"
    }"#;

    const EXAMPLE_TRACE_SUICIDE: &str = r#"{
        "action": {
            "address": "0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            "refundAddress": "0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359",
            "balance": "0x0"
        },
        "blockHash": "0x6474a53a9ebf72d306a1406ec12ded12e210b6c3141b4373bfb3a3cea987dfb8",
        "blockNumber": 988775,
        "result": {
            "gasUsed": "0x4b419",
            "output": "0x0000000000000000000000000000000000000000000000000000000000000000"
        },
        "subtraces": 1,
        "traceAddress": [],
        "transactionHash": "0x342c284238149db221f9d87db87f90ffad7ac0aac57c0c480142f4c21b63f652",
        "transactionPosition": 1,
        "type": "suicide"
    }"#;

    const EXAMPLE_TRACE_REWARD: &str = r#"{
        "action": {
            "author": "0xd1220a0cf47c7b9be7a2e6ba89f429762e7b9adb",
            "value": "0x0",
            "rewardType": "block"
        },
        "blockHash": "0x6474a53a9ebf72d306a1406ec12ded12e210b6c3141b4373bfb3a3cea987dfb8",
        "blockNumber": 988775,
        "result": {
            "gasUsed": "0x4b419",
            "output": "0x0000000000000000000000000000000000000000000000000000000000000000"
        },
        "subtraces": 1,
        "traceAddress": [],
        "transactionHash": "0x342c284238149db221f9d87db87f90ffad7ac0aac57c0c480142f4c21b63f652",
        "transactionPosition": 1,
        "type": "reward"
    }"#;

    #[test]
    fn test_deserialize_trace() {
        let _trace: Trace = serde_json::from_str(EXAMPLE_TRACE_CALL).unwrap();
        let _trace: Trace = serde_json::from_str(EXAMPLE_TRACE_CREATE).unwrap();
        let _trace: Trace = serde_json::from_str(EXAMPLE_TRACE_SUICIDE).unwrap();
        let _trace: Trace = serde_json::from_str(EXAMPLE_TRACE_REWARD).unwrap();
    }
}
