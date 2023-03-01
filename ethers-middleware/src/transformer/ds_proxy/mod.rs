pub mod factory;
use factory::{CreatedFilter, DsProxyFactory, ADDRESS_BOOK};

use super::{Transformer, TransformerError};
use ethers_contract::{builders::ContractCall, BaseContract, ContractError};
use ethers_core::{
    abi::parse_abi,
    types::{transaction::eip2718::TypedTransaction, *},
    utils::id,
};
use ethers_providers::Middleware;
use std::sync::Arc;

/// The function signature of DsProxy's execute function, to execute data on a target address.
const DS_PROXY_EXECUTE_TARGET: &str =
    "function execute(address target, bytes memory data) public payable returns (bytes memory response)";
/// The function signature of DsProxy's execute function, to deploy bytecode and execute data on it.
const DS_PROXY_EXECUTE_CODE: &str =
    "function execute(bytes memory code, bytes memory data) public payable returns (address target, bytes memory response)";

/// Represents the DsProxy type that implements the [Transformer](super::Transformer) trait.
///
/// # Example
///
/// ```no_run
/// use ethers_middleware::{SignerMiddleware, transformer::DsProxy};
/// use ethers_signers::LocalWallet;
/// use ethers_providers::{Provider, Http};
/// use ethers_core::types::{Address, Bytes};
/// use std::{convert::TryFrom, sync::Arc};
///
/// type HttpWallet = SignerMiddleware<Provider<Http>, LocalWallet>;
///
/// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// // instantiate client that can sign transactions.
/// let wallet: LocalWallet = "380eb0f3d505f087e438eca80bc4df9a7faa24f868e69fc0440261a0fc0567dc"
///     .parse()?;
/// let provider = Provider::<Http>::try_from("http://localhost:8545")?;
/// let client = SignerMiddleware::new(provider, wallet);
///
/// # let ds_proxy_addr = Address::random();
/// // instantiate DsProxy by providing its address.
/// let ds_proxy = DsProxy::new(ds_proxy_addr);
///
/// // execute a transaction via the DsProxy instance.
/// let target = Address::random();
/// let calldata: Bytes = vec![0u8; 32].into();
/// let contract_call = ds_proxy.execute::<HttpWallet, Arc<HttpWallet>, Address>(
///     Arc::new(client),
///     target,
///     calldata,
/// )?;
/// let pending_tx = contract_call.send().await?;
/// let _tx_receipt = pending_tx.await?;
///
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
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

    /// The address of the DsProxy instance.
    pub fn address(&self) -> Address {
        self.address
    }
}

impl DsProxy {
    /// Deploys a new DsProxy contract to the Ethereum network.
    pub async fn build<M: Middleware, C: Into<Arc<M>>>(
        client: C,
        factory: Option<Address>,
        owner: Address,
    ) -> Result<Self, ContractError<M>> {
        let client = client.into();

        // Fetch chain id and the corresponding address of DsProxyFactory contract
        // preference is given to DsProxyFactory contract's address if provided
        // otherwise check the address book for the client's chain ID.
        let factory: Address = match factory {
            Some(addr) => addr,
            None => {
                let chain_id =
                    client.get_chainid().await.map_err(ContractError::from_middleware_error)?;
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
            .legacy()
            .send()
            .await?
            .await?
            .ok_or(ContractError::ContractNotDeployed)?;

        // decode the event log to get the address of the deployed contract.
        if tx_receipt.status == Some(U64::from(1u64)) {
            // fetch the appropriate log. Only one event is logged by the DsProxyFactory contract,
            // the others are logged by the deployed DsProxy contract and hence can be ignored.
            let log = tx_receipt
                .logs
                .iter()
                .find(|i| i.address == factory)
                .ok_or(ContractError::ContractNotDeployed)?;

            // decode the log.
            let created_filter: CreatedFilter =
                ds_proxy_factory.decode_event("Created", log.topics.clone(), log.data.clone())?;

            // instantiate the ABI and return.
            let contract = parse_abi(&[DS_PROXY_EXECUTE_TARGET, DS_PROXY_EXECUTE_CODE])
                .expect("could not parse ABI")
                .into();
            Ok(Self { address: created_filter.proxy, contract })
        } else {
            Err(ContractError::ContractNotDeployed)
        }
    }
}

impl DsProxy {
    /// Execute a tx through the DsProxy instance. The target can either be a deployed smart
    /// contract's address, or bytecode of a compiled smart contract. Depending on the target, the
    /// appropriate `execute` method is called, that is, either
    /// [execute(address,bytes)](https://github.com/dapphub/ds-proxy/blob/master/src/proxy.sol#L53-L58)
    /// or [execute(bytes,bytes)](https://github.com/dapphub/ds-proxy/blob/master/src/proxy.sol#L39-L42).
    pub fn execute<M: Middleware, C: Into<Arc<M>>, T: Into<AddressOrBytes>>(
        &self,
        client: C,
        target: T,
        data: Bytes,
    ) -> Result<ContractCall<M, Bytes>, ContractError<M>> {
        // construct the full contract using DsProxy's address and the injected client.
        let ds_proxy = self.contract.clone().into_contract(self.address, client.into());

        match target.into() {
            // handle the case when the target is an address to a deployed contract.
            AddressOrBytes::Address(addr) => {
                let selector = id("execute(address,bytes)");
                let args = (addr, data);
                Ok(ds_proxy.method_hash(selector, args)?)
            }
            // handle the case when the target is actually bytecode of a contract to be deployed
            // and executed on.
            AddressOrBytes::Bytes(code) => {
                let selector = id("execute(bytes,bytes)");
                let args = (code, data);
                Ok(ds_proxy.method_hash(selector, args)?)
            }
        }
    }
}

impl Transformer for DsProxy {
    fn transform(&self, tx: &mut TypedTransaction) -> Result<(), TransformerError> {
        // the target address cannot be None.
        let target =
            *tx.to_addr().ok_or_else(|| TransformerError::MissingField("to".to_string()))?;

        // fetch the data field.
        let data = tx.data().cloned().unwrap_or_else(|| vec![].into());

        // encode data as the ABI encoded data for DSProxy's execute method.
        let selector = id("execute(address,bytes)");
        let encoded_data = self.contract.encode_with_selector(selector, (target, data))?;

        // update appropriate fields of the proxy tx.
        tx.set_data(encoded_data);
        tx.set_to(self.address);

        Ok(())
    }
}
