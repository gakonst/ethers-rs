use super::{ProxyWallet, ProxyWalletError};
use ethers_contract::BaseContract;
use ethers_core::{
    abi::{parse_abi, Tokenize},
    types::*,
};

const DS_PROXY_EXECUTE_SIGNATURE: &str =
    "function execute(address target, bytes data) public payable returns (bytes memory response)";

#[derive(Debug)]
pub struct DsProxy {
    address: Address,
}

impl DsProxy {
    /// Create a new instance of DsProxy by providing the address of the DsProxy contract that has
    /// already been deployed to the Ethereum network.
    pub fn new(address: Address) -> Self {
        Self { address }
    }
}

impl ProxyWallet for DsProxy {
    fn get_proxy_tx(&self, tx: TransactionRequest) -> Result<TransactionRequest, ProxyWalletError> {
        // clone the tx into a new proxy tx.
        let mut proxy_tx = tx.clone();

        // the target address cannot be None.
        let mut target = Address::default();
        if let Some(NameOrAddress::Address(addr)) = tx.to {
            target = addr;
        } else {
            return Err(ProxyWalletError::Dummy);
        }

        // fetch the data field.
        let data = tx.data.unwrap_or(vec![].into());

        // encode data as the ABI encoded data for DSProxy's execute method.
        let ds_proxy_base: BaseContract = parse_abi(&[DS_PROXY_EXECUTE_SIGNATURE])?.into();
        let encoded_data = ds_proxy_base.encode("execute", (target, data))?;

        // update appropriate fields of the proxy tx.
        proxy_tx.data = Some(encoded_data);
        proxy_tx.to = Some(NameOrAddress::Address(self.address));

        Ok(proxy_tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex::FromHex;

    #[test]
    fn test_get_proxy_tx() {
        let ds_proxy_addr = Address::random();
        let ds_proxy = DsProxy::new(ds_proxy_addr);

        let tx: TransactionRequest = serde_json::from_str(
            r#"{
            "gas":"0xc350",
            "gasPrice":"0x4a817c800",
            "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
            "data":"0x68656c6c6f21",
            "nonce":"0x15",
            "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
            "transactionIndex":"0x41",
            "value":"0xf3dbb76162000",
            "chain_id": "0x1"
        }"#,
        )
        .unwrap();

        let proxy_tx = ds_proxy.get_proxy_tx(tx).unwrap();
        let encoded_data = Vec::from_hex(
            "1cff79cd000000000000000000000000f02c1c8e6114b1dbe8937a39260b5b0a374432bb\
            0000000000000000000000000000000000000000000000000000000000000040000000000\
            000000000000000000000000000000000000000000000000000000668656c6c6f21000000\
            0000000000000000000000000000000000000000000000",
        )
        .unwrap()
        .into();
        assert_eq!(proxy_tx.data, Some(encoded_data));
        assert_eq!(proxy_tx.to, Some(NameOrAddress::Address(ds_proxy_addr)));
    }
}
