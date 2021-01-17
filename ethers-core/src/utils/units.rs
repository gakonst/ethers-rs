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
            Units::Wei => 1,
            Units::Other(inner) => *inner,
        }
    }
}

impl From<u32> for Units {
    fn from(src: u32) -> Self {
        Units::Other(src)
    }
}

impl From<i32> for Units {
    fn from(src: i32) -> Self {
        Units::Other(src as u32)
    }
}

impl From<usize> for Units {
    fn from(src: usize) -> Self {
        Units::Other(src as u32)
    }
}

impl From<&str> for Units {
    fn from(src: &str) -> Self {
        match src.to_lowercase().as_str() {
            "ether" => Units::Ether,
            "gwei" => Units::Gwei,
            "wei" => Units::Wei,
            _ => panic!("unrecognized units"),
        }
    }
}
