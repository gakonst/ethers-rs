use super::{Transformer, TransformerError};
use ethers_contract::BaseContract;
use ethers_core::{abi::parse_abi, types::*};

const GNOSIS_SAFE_EXEC_TRANSACTION: &str = "function execTransaction(\
    address to, \
    uint256 value, \
    bytes calldata data, \
    uint8 operation, \
    uint256 safeTxGas, \
    uint256 baseGas, \
    uint256 gasPrice, \
    address gasToken, \
    address payable refundReceiver, \
    bytes calldata signatures \
)";

#[derive(Debug)]
pub struct GnosisSafe {
    address: Address,
    contract: BaseContract,
}

impl GnosisSafe {
    /// Create a new instance of GnosisSafe by providing the address of the GnosisSafeProxy that
    /// has already been deployed to the Ethereum network.
    pub fn new(address: Address) -> Self {
        let contract = parse_abi(&[GNOSIS_SAFE_EXEC_TRANSACTION])
            .expect("could not parse ABI")
            .into();

        Self { address, contract }
    }
}

impl Transformer for GnosisSafe {
    fn transform(&self, tx: TransactionRequest) -> Result<TransactionRequest, TransformerError> {
        let mut proxy_tx = tx.clone();

        // TODO: update tx

        Ok(proxy_tx)
    }
}
