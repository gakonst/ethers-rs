use super::ConversionError;
use std::{convert::TryFrom, fmt, str::FromStr};

/// Common Ethereum unit types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Units {
    /// Wei is equivalent to 1 wei.
    Wei,
    /// Gwei is equivalent to 1e9 wei.
    Gwei,
    /// Ether is equivalent to 1e18 wei.
    Ether,
    /// Other less frequent unit sizes, equivalent to 1e{0} wei.
    Other(u32),
}

impl fmt::Display for Units {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_num().to_string().as_str())
    }
}

impl TryFrom<u32> for Units {
    type Error = ConversionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Units::Other(value))
    }
}

impl TryFrom<i32> for Units {
    type Error = ConversionError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Ok(Units::Other(value as u32))
    }
}

impl TryFrom<usize> for Units {
    type Error = ConversionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Units::Other(value as u32))
    }
}

impl TryFrom<String> for Units {
    type Error = ConversionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value)
    }
}

impl<'a> TryFrom<&'a String> for Units {
    type Error = ConversionError;

    fn try_from(value: &'a String) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl TryFrom<&str> for Units {
    type Error = ConversionError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl FromStr for Units {
    type Err = ConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "eth" | "ether" => Units::Ether,
            "gwei" | "nano" | "nanoether" => Units::Gwei,
            "wei" => Units::Wei,
            _ => return Err(ConversionError::UnrecognizedUnits(s.to_string())),
        })
    }
}

impl From<Units> for u32 {
    fn from(units: Units) -> Self {
        units.as_num()
    }
}

impl From<Units> for i32 {
    fn from(units: Units) -> Self {
        units.as_num() as i32
    }
}

impl From<Units> for usize {
    fn from(units: Units) -> Self {
        units.as_num() as usize
    }
}

impl Units {
    pub fn as_num(&self) -> u32 {
        match self {
            Units::Wei => 0,
            Units::Gwei => 9,
            Units::Ether => 18,
            Units::Other(inner) => *inner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Units::*;

    #[test]
    fn test_units() {
        assert_eq!(Wei.as_num(), 0);
        assert_eq!(Gwei.as_num(), 9);
        assert_eq!(Ether.as_num(), 18);
        assert_eq!(Other(10).as_num(), 10);
        assert_eq!(Other(20).as_num(), 20);
    }

    #[test]
    fn test_into() {
        assert_eq!(Units::try_from("wei").unwrap(), Wei);
        assert_eq!(Units::try_from("gwei").unwrap(), Gwei);
        assert_eq!(Units::try_from("ether").unwrap(), Ether);

        assert_eq!(Units::try_from("wei".to_string()).unwrap(), Wei);
        assert_eq!(Units::try_from("gwei".to_string()).unwrap(), Gwei);
        assert_eq!(Units::try_from("ether".to_string()).unwrap(), Ether);

        assert_eq!(Units::try_from(&"wei".to_string()).unwrap(), Wei);
        assert_eq!(Units::try_from(&"gwei".to_string()).unwrap(), Gwei);
        assert_eq!(Units::try_from(&"ether".to_string()).unwrap(), Ether);
    }
}
