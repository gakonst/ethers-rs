use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/4byte.go#L50
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FourByteFrame(pub BTreeMap<String, u64>);
