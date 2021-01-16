use super::{ProxyWallet, ProxyWalletError};
use ethers_contract::BaseContract;
use ethers_core::{abi::parse_abi, types::*};

#[derive(Debug)]
pub struct GnosisSafe {
    address: Address,
}

impl GnosisSafe {
    /// Create a new instance of GnosisSafe by providing the address of the GnosisSafeProxy that
    /// has already been deployed to the Ethereum network.
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

impl ProxyWallet for GnosisSafe {
    fn get_proxy_tx(&self, tx: TransactionRequest) -> Result<TransactionRequest, ProxyWalletError> {
        let mut proxy_tx = tx.clone();

        // TODO: update the `data` field of the transaction to be the ABI encoded data for
        // GnosisSafe's method.
        let _gnosis_safe_base: BaseContract = parse_abi(&["function execTransaction( \
                address to, \
                uint256 value, \
                bytes calldata data, \
                Enum.Operation operation, \
                uint256 safeTxGas, \
                uint256 baseGas, \
                uint256 gasPrice, \
                address gasToken, \
                address payable refundReceiver, \
                bytes calldata signatures \
            )"])?
        .into();

        // update the `to` field of the transaction to be the address of the proxy wallet.
        proxy_tx.to = Some(NameOrAddress::Address(self.address));

        Ok(proxy_tx)
    }
}
