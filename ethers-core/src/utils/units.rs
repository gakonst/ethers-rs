use super::ConversionError;

/// Common Ethereum unit types.
pub enum Units {
    /// Ether corresponds to 1e18 Wei
    Ether,
    /// Gwei corresponds to 1e9 Wei
    Gwei,
    /// Wei corresponds to 1 Wei
    Wei,
    /// Use this for other less frequent unit sizes
    Other(u32),
}

impl Units {
    pub fn as_num(&self) -> u32 {
        match self {
            Units::Ether => 18,
            Units::Gwei => 9,
            Units::Wei => 0,
            Units::Other(inner) => *inner,
        }
    }
}

use std::convert::TryFrom;

impl TryFrom<u32> for Units {
    type Error = ConversionError;

    fn try_from(src: u32) -> Result<Self, Self::Error> {
        Ok(Units::Other(src))
    }
}

impl TryFrom<i32> for Units {
    type Error = ConversionError;

    fn try_from(src: i32) -> Result<Self, Self::Error> {
        Ok(Units::Other(src as u32))
    }
}

impl TryFrom<usize> for Units {
    type Error = ConversionError;

    fn try_from(src: usize) -> Result<Self, Self::Error> {
        Ok(Units::Other(src as u32))
    }
}

impl std::convert::TryFrom<&str> for Units {
    type Error = ConversionError;

    fn try_from(src: &str) -> Result<Self, Self::Error> {
        Ok(match src.to_lowercase().as_str() {
            "ether" => Units::Ether,
            "gwei" => Units::Gwei,
            "wei" => Units::Wei,
            _ => return Err(ConversionError::UnrecognizedUnits(src.to_string())),
        })
    }
}
