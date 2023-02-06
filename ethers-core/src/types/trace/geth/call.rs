use crate::{
    types::{Address, Bytes, NameOrAddress, H256, U256},
    utils::from_int_or_hex,
};
use serde::{Deserialize, Serialize};

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/call.go#L37
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallFrame {
    #[serde(rename = "type")]
    pub typ: String,
    pub from: Address,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,
    #[serde(default, deserialize_with = "from_int_or_hex")]
    pub gas: U256,
    #[serde(default, deserialize_with = "from_int_or_hex", rename = "gasUsed")]
    pub gas_used: U256,
    pub input: Bytes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<Bytes>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calls: Option<Vec<CallFrame>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<CallLogFrame>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CallLogFrame {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    address: Option<Address>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    topics: Option<Vec<H256>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    data: Option<Bytes>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub only_top_call: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub with_log: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    // See <https://github.com/ethereum/go-ethereum/tree/master/eth/tracers/internal/tracetest/testdata>
    const DEFAULT: &str = include!("./test_data/call_tracer/default.rs");
    const LEGACY: &str = include!("./test_data/call_tracer/legacy.rs");
    const ONLY_TOP_CALL: &str = include!("./test_data/call_tracer/only_top_call.rs");
    const WITH_LOG: &str = include!("./test_data/call_tracer/with_log.rs");

    #[test]
    fn test_serialize_call_trace() {
        let mut opts = GethDebugTracingCallOptions::default();
        opts.tracing_options.disable_storage = Some(false);
        opts.tracing_options.tracer =
            Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::CallTracer));
        opts.tracing_options.tracer_config =
            Some(GethDebugTracerConfig::BuiltInTracer(GethDebugBuiltInTracerConfig::CallTracer(
                CallConfig { only_top_call: Some(true), with_log: Some(true) },
            )));

        assert_eq!(
            serde_json::to_string(&opts).unwrap(),
            r#"{"disableStorage":false,"tracer":"callTracer","tracerConfig":{"onlyTopCall":true,"withLog":true}}"#
        );
    }

    #[test]
    fn test_deserialize_call_trace() {
        let _trace: CallFrame = serde_json::from_str(DEFAULT).unwrap();
        let _trace: CallFrame = serde_json::from_str(LEGACY).unwrap();
        let _trace: CallFrame = serde_json::from_str(ONLY_TOP_CALL).unwrap();
        let _trace: CallFrame = serde_json::from_str(WITH_LOG).unwrap();
    }
}
