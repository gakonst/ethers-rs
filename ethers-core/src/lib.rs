#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(unused_crate_dependencies)]

pub mod types;

pub mod abi;

/// Various utilities
pub mod utils;

#[cfg(feature = "macros")]
pub mod macros;

// re-export rand to avoid potential confusion when there's rand version mismatches
pub use rand;

// re-export k256
pub use k256;
