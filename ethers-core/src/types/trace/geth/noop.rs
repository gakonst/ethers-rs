use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/noop.go#L35
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoopFrame(BTreeMap<Null, Null>);
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
struct Null;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    const DEFAULT: &str = r#"{}"#;

    #[test]
    fn test_serialize_noop_trace() {
        let mut opts = GethDebugTracingCallOptions::default();
        opts.tracing_options.tracer =
            Some(GethDebugTracerType::BuiltInTracer(GethDebugBuiltInTracerType::NoopTracer));

        assert_eq!(serde_json::to_string(&opts).unwrap(), r#"{"tracer":"noopTracer"}"#);
    }

    #[test]
    fn test_deserialize_noop_trace() {
        let _trace: NoopFrame = serde_json::from_str(DEFAULT).unwrap();
    }
}
