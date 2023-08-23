use std::result::Result as StdResult;

/// The Multicall contract bindings. Auto-generated with `abigen`.
pub mod contract;

pub mod constants;

if_providers! {
    mod middleware;
    pub use middleware::{Call, Multicall, MulticallContract, Result};

    pub mod error;
}

/// The version of the [`Multicall`].
/// Used to determine which methods of the Multicall smart contract to use:
/// - [`Multicall`] : `aggregate((address,bytes)[])`
/// - [`Multicall2`] : `try_aggregate(bool, (address,bytes)[])`
/// - [`Multicall3`] : `aggregate3((address,bool,bytes)[])` or
///   `aggregate3Value((address,bool,uint256,bytes)[])`
///
/// [`Multicall`]: #variant.Multicall
/// [`Multicall2`]: #variant.Multicall2
/// [`Multicall3`]: #variant.Multicall3
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum MulticallVersion {
    /// V1
    Multicall = 1,
    /// V2
    Multicall2 = 2,
    /// V3
    #[default]
    Multicall3 = 3,
}

impl From<MulticallVersion> for u8 {
    fn from(v: MulticallVersion) -> Self {
        v as u8
    }
}

impl TryFrom<u8> for MulticallVersion {
    type Error = String;
    fn try_from(v: u8) -> StdResult<Self, Self::Error> {
        match v {
            1 => Ok(MulticallVersion::Multicall),
            2 => Ok(MulticallVersion::Multicall2),
            3 => Ok(MulticallVersion::Multicall3),
            _ => Err(format!("Invalid Multicall version: {v}. Accepted values: 1, 2, 3.")),
        }
    }
}

impl MulticallVersion {
    /// True if call is v1
    #[inline]
    pub fn is_v1(&self) -> bool {
        matches!(self, Self::Multicall)
    }

    /// True if call is v2
    #[inline]
    pub fn is_v2(&self) -> bool {
        matches!(self, Self::Multicall2)
    }

    /// True if call is v3
    #[inline]
    pub fn is_v3(&self) -> bool {
        matches!(self, Self::Multicall3)
    }
}
