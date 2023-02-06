use crate::types::{Address, H256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/prestate.go#L36
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum PreStateFrame {
    Default(PreStateMode),
    Diff(DiffMode),
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreStateMode(pub BTreeMap<Address, AccountState>);

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiffMode {
    pub pre: BTreeMap<Address, AccountState>,
    pub post: BTreeMap<Address, AccountState>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<BTreeMap<H256, H256>>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreStateConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub diff_mode: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    // See <https://github.com/ethereum/go-ethereum/tree/master/eth/tracers/internal/tracetest/testdata>
    const DEFAULT: &str = include!("./test_data/pre_state_tracer/default.rs");
    const LEGACY: &str = include!("./test_data/pre_state_tracer/legacy.rs");
    const DIFF_MODE: &str = include!("./test_data/pre_state_tracer/diff_mode.rs");

    #[test]
    fn test_serialize_pre_state_trace() {
        let mut opts = GethDebugTracingCallOptions::default();
        opts.tracing_options.disable_storage = Some(false);
        opts.tracing_options.tracer =
            Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::PreStateTracer));
        opts.tracing_options.tracer_config = Some(GethDebugTracerConfig::BuiltInTracer(
            GethDebugBuiltInTracerConfig::PreStateTracer(PreStateConfig { diff_mode: Some(true) }),
        ));

        assert_eq!(
            serde_json::to_string(&opts).unwrap(),
            r#"{"disableStorage":false,"tracer":"prestateTracer","tracerConfig":{"diffMode":true}}"#
        );
    }

    #[test]
    fn test_deserialize_pre_state_trace() {
        let _trace: PreStateFrame = serde_json::from_str(DEFAULT).unwrap();
        let _trace: PreStateFrame = serde_json::from_str(LEGACY).unwrap();
        let _trace: PreStateFrame = serde_json::from_str(DIFF_MODE).unwrap();
    }
}
