use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// https://github.com/ethereum/go-ethereum/blob/a9ef135e2dd53682d106c6a2aede9187026cc1de/eth/tracers/native/noop.go#L35
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoopFrame(BTreeMap<Null, Null>);
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
struct Null;
