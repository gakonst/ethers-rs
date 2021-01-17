mod factory;
use factory::{CreatedFilter, DsProxyFactory, ADDRESS_BOOK};

use super::{Transformer, TransformerError};
use ethers_contract::{BaseContract, ContractError};
use ethers_core::{abi::parse_abi, types::*};
use ethers_providers::Middleware;
use ethers_signers::Signer;
use std::sync::Arc;

/// The function signature of DsProxy's execute function, to execute data on a target address.
const DS_PROXY_EXECUTE_TARGET: &str =
    "function execute(address target, bytes memory data) public payable returns (bytes memory response)";
/// The function signature of DsProxy's execute function, to deploy bytecode and execute data on it.
const DS_PROXY_EXECUTE_CODE: &str =
    "function execute(bytes memory code, bytes memory data) public payable returns (address target, bytes memory response)";

#[derive(Debug)]
/// Represents the DsProxy type that implements the [Transformer](super::Transformer) trait.
pub struct DsProxy {
    address: Address,
    contract: BaseContract,
}

impl DsProxy {
    /// Create a new instance of DsProxy by providing the address of the DsProxy contract that has
    /// already been deployed to the Ethereum network.
    pub fn new(address: Address) -> Self {
        let contract = parse_abi(&[DS_PROXY_EXECUTE_TARGET, DS_PROXY_EXECUTE_CODE])
            .expect("could not parse ABI")
            .into();

        Self { address, contract }
    }
}

impl DsProxy {
    /// Deploys a new DsProxy contract to the Ethereum network.
    pub async fn build<M: Middleware + Signer, C: Into<Arc<M>>>(
        client: C,
        factory: Option<Address>,
        owner: Option<Address>,
    ) -> Result<Self, ContractError<M>> {
        let client = client.into();
        let owner = owner.unwrap_or(client.address());

        // Fetch chain id and the corresponding address of DsProxyFactory contract
        // preference is given to DsProxyFactory contract's address if provided
        // otherwise check the address book for the client's chain ID.
        let factory: Address = match factory {
            Some(addr) => addr,
            None => {
                let chain_id = client
                    .get_chainid()
                    .await
                    .map_err(ContractError::MiddlewareError)?;
                match ADDRESS_BOOK.get(&chain_id) {
                    Some(addr) => *addr,
                    None => panic!(
                        "Must either be a supported Network ID or provide DsProxyFactory contract address"
                    ),
                }
            }
        };

        // broadcast the tx to deploy a new DsProxy.
        let ds_proxy_factory = DsProxyFactory::new(factory, client);
        let tx_receipt = ds_proxy_factory
            .build(owner)
            .send()
            .await?
            .await
            .map_err(ContractError::ProviderError)?;

        // decode the event log to get the address of the deployed contract.
        if tx_receipt.status == Some(U64::from(1u64)) {
            let log: CreatedFilter = ds_proxy_factory.decode_event(
                "Created",
                tx_receipt.logs[0].topics.clone(),
                tx_receipt.logs[0].data.clone(),
            )?;

            let contract = parse_abi(&[DS_PROXY_EXECUTE_TARGET, DS_PROXY_EXECUTE_CODE])
                .expect("could not parse ABI")
                .into();

            Ok(Self {
                address: log.proxy,
                contract,
            })
        } else {
            Err(ContractError::ContractNotDeployed)
        }
    }
}

impl Transformer for DsProxy {
    fn transform(&self, tx: TransactionRequest) -> Result<TransactionRequest, TransformerError> {
        // clone the tx into a new proxy tx.
        let mut proxy_tx = tx.clone();

        // the target address cannot be None.
        let target = match tx.to {
            Some(NameOrAddress::Address(addr)) => Ok(addr),
            _ => Err(TransformerError::Dummy),
        }?;

        // fetch the data field.
        let data = tx.data.unwrap_or(vec![].into());

        // encode data as the ABI encoded data for DSProxy's execute method.
        let encoded_data = self.contract.encode("execute", (target, data))?;

        // update appropriate fields of the proxy tx.
        proxy_tx.data = Some(encoded_data);
        proxy_tx.to = Some(NameOrAddress::Address(self.address));

        Ok(proxy_tx)
    }
}
