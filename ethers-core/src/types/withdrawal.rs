use crate::types::{Address, U256, U64};
use serde::{Deserialize, Serialize};

/// A validator withdrawal from the consensus layer.
/// See EIP-4895: Beacon chain push withdrawals as operations.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Withdrawal {
    /// Monotonically increasing identifier issued by consensus layer
    pub index: U64,

    /// Index of validator associated with withdrawal
    #[serde(rename = "validatorIndex")]
    pub validator_index: U64,

    /// Target address for withdrawn ether
    pub address: Address,

    /// Value of withdrawal (in wei)
    pub amount: U256,
}

impl rlp::Encodable for Withdrawal {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        s.begin_list(4);
        s.append(&self.index);
        s.append(&self.validator_index);
        s.append(&self.address);
        s.append(&self.amount);
    }
}
