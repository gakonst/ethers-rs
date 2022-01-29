use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, Block, BlockId, BlockNumber, Bytes,
    NameOrAddress, TransactionReceipt, TxHash, H256, U256, U64,
};
use ethers_providers::{
    maybe, FromErr, JsonRpcClient, Middleware, PendingTransaction, PendingTxState, ProviderError,
};
use evm_adapters::{
    sputnik::{Executor, SputnikExecutor},
    Evm, EvmError,
};
use sputnik::backend::Backend;
use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use thiserror::Error;
use tokio::sync::RwLock;

const DEFAULT_SENDER: &'static str = "0xD3D13a578a53685B4ac36A1Bab31912D2B2A2F36";

pub trait VmShow {
    fn gas_price(&self) -> U256;
    fn block_number(&self) -> U256;
    fn chain_id(&self) -> U256;
    fn balance(&self, from: Address) -> U256;
    fn gas_limit(&self) -> U256;
    fn block_hash(&self, num: U256) -> H256;
}

impl<'a, S, E> VmShow for Executor<S, E>
where
    E: SputnikExecutor<S>,
    S: Backend,
{
    fn gas_price(&self) -> U256 {
        self.executor.state().gas_price()
    }
    fn block_number(&self) -> U256 {
        self.executor.state().block_number()
    }
    fn chain_id(&self) -> U256 {
        self.executor.state().block_number()
    }
    fn balance(&self, addr: Address) -> U256 {
        self.executor.state().basic(addr).balance
    }
    fn gas_limit(&self) -> U256 {
        self.executor.state().block_gas_limit()
    }
    fn block_hash(&self, num: U256) -> H256 {
        self.executor.state().block_hash(num)
    }
}

#[derive(Clone)]
pub struct Forge<M, E, S> {
    pub vm: Arc<RwLock<E>>,
    inner: M,
    _ghost: PhantomData<S>,
}

pub enum TxOutput {
    CallRes(Bytes),
    CreateRes(Address),
}

impl<M, E, S> Forge<M, E, S> {
    pub fn new(inner: M, vm: Arc<RwLock<E>>) -> Self {
        Self { vm, inner, _ghost: PhantomData }
    }
    async fn vm(&self) -> impl Deref<Target = E> + '_ {
        self.vm.read().await
    }
    async fn vm_mut(&self) -> impl DerefMut<Target = E> + '_ {
        self.vm.write().await
    }
}

// Gives some structure to the result of Evm::call_raw()
struct TxRes<Ex> {
    pub output: TxOutput,
    pub exit: Ex,
    pub gas: u64,
    pub logs: Vec<String>,
}
impl<M, E, S> Forge<M, E, S>
where
    Self: Middleware,
    E: Evm<S> + VmShow,
    <Self as Middleware>::Error: From<eyre::ErrReport>,
{
    //TODO: incoporate block parameter
    async fn apply_tx(
        &self,
        tx: &TypedTransaction,
    ) -> Result<TxRes<E::ReturnReason>, <Self as Middleware>::Error> {
        // Pull fields from tx to pass to evm
        let default_from = DEFAULT_SENDER.parse().unwrap();
        let default_val = U256::zero();

        let from = tx.from().unwrap_or(&default_from);
        let maybe_to = tx.to().map(|id| async move {
            match id {
                NameOrAddress::Name(ens) => self.resolve_name(ens).await,
                NameOrAddress::Address(addr) => Ok(*addr),
            }
        });
        let data = tx.data().map_or(Default::default(), |d| d.clone());
        let val = tx.value().unwrap_or(&default_val);

        if let Some(fut) = maybe_to {
            // (contract) call
            let to = fut.await?;
            let (bytes, exit, gas, logs) =
                self.vm_mut().await.call_raw(*from, to, data, *val, false)?;
            Ok(TxRes { output: TxOutput::CallRes(bytes), exit, gas, logs })
        } else {
            // contract deployment
            let (addr, exit, gas, logs) = self.vm_mut().await.deploy(*from, data.clone(), *val)?;
            Ok(TxRes { output: TxOutput::CreateRes(addr), exit, gas, logs })
        }
    }

    async fn is_latest(&self, id: BlockId) -> Result<bool, <Self as Middleware>::Error> {
        match id {
            BlockId::Hash(hash) => {
                let vm = self.vm().await;
                // let last_num = vm.block_number() - U256::one();
                let last_hash = vm.block_hash(vm.block_number() - U256::one());
                // If we get the default hash back, the vm doesn't have the block data
                Ok(last_hash != Default::default() && hash == last_hash)
            }
            BlockId::Number(n) => match n {
                BlockNumber::Latest => Ok(true),
                BlockNumber::Number(num) => {
                    Ok(num == (self.get_block_number().await? - U64::one()))
                }
                // BlockNumber::Pending => Ok(false), //TODO
                _ => Ok(false),
            },
        }
    }

    // Sputnik can provide hashes for any block it produced, but not the rest of the block data
    async fn get_block_hash(&self, num: U256) -> H256 {
        todo!()
    }

    async fn to_addr<T: Into<NameOrAddress>>(
        &self,
        id: T,
    ) -> Result<Address, <Self as Middleware>::Error> {
        match id.into() {
            NameOrAddress::Name(ref ens) => self.resolve_name(ens).await,
            NameOrAddress::Address(a) => Ok(a),
        }
    }
}

// TODO: Stand-in impl because some sputnik component is not Debug
impl<M, E, S> Debug for Forge<M, E, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forge").finish()
    }
}

#[derive(Error, Debug)]
pub enum ForgeError<M: Middleware> {
    #[error("{0}")]
    MiddlewareError(M::Error),
    #[error("{0}")]
    ProviderError(ProviderError),
    #[error("{0}")]
    EvmError(EvmError),
    #[error("{0}")]
    Eyre(eyre::Error),
}
impl<M: Middleware> FromErr<M::Error> for ForgeError<M> {
    fn from(src: M::Error) -> ForgeError<M> {
        ForgeError::MiddlewareError(src)
    }
}
impl<M> From<ProviderError> for ForgeError<M>
where
    M: Middleware,
{
    fn from(src: ProviderError) -> Self {
        Self::ProviderError(src)
    }
}
impl<M> From<EvmError> for ForgeError<M>
where
    M: Middleware,
{
    fn from(src: EvmError) -> Self {
        Self::EvmError(src)
    }
}
impl<M> From<eyre::ErrReport> for ForgeError<M>
where
    M: Middleware,
{
    fn from(src: eyre::ErrReport) -> Self {
        Self::EvmError(EvmError::Eyre(src))
    }
}

#[async_trait]
impl<M, E, S> Middleware for Forge<M, E, S>
where
    M: Middleware,
    E: Evm<S> + VmShow + Send + Sync,
    S: Clone + Send + Sync + Debug,
    E::ReturnReason: Send,
{
    type Error = ForgeError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &Self::Inner {
        &self.inner
    }

    async fn estimate_gas(&self, _tx: &TypedTransaction) -> Result<U256, Self::Error> {
        Ok(self.vm().await.gas_limit())
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        Ok(self.vm().await.gas_price())
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        Ok(self.vm().await.block_number().as_u64().into())
    }

    async fn get_block<T: Into<BlockId> + Send + Sync>(
        &self,
        id: T,
    ) -> Result<Option<Block<TxHash>>, Self::Error> {
        let id = id.into();
        if self.is_latest(id).await? {
            // TODO: don't do a very good job of reconstructing block data here.
            // Try to reconstruct as much of the block from evm data as possible.
            let mut block: Block<TxHash> = Default::default();
            let vm = self.vm().await;
            let num = vm.block_number() - U256::one();
            block.number = Some(num.as_u64().into());
            block.hash = Some(vm.block_hash(num));
            block.parent_hash = self.get_block_hash(num - U256::one()).await;
            Ok(Some(block))
        } else {
            self.inner().get_block(id).await.map_err(FromErr::from)
        }
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        Ok(self.vm().await.chain_id())
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        if block.is_none() || self.is_latest(block.unwrap()).await? {
            let addr = self.to_addr(from).await?;
            Ok(self.vm().await.balance(addr))
        } else {
            self.inner.get_balance(from, block).await.map_err(FromErr::from)
        }
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx = tx.into();
        self.fill_transaction(&mut tx, block).await?;

        // run the tx
        let res = self.apply_tx(&tx).await?;

        // create receipt and populate with result of applying the tx
        let mut receipt = TransactionReceipt::default();
        receipt.gas_used = Some(res.gas.into());
        receipt.status = Some((if E::is_success(&res.exit) { 1usize } else { 0 }).into());
        if let TxOutput::CreateRes(addr) = res.output {
            receipt.contract_address = Some(addr);
        }

        // Fake the tx hash for the receipt. Should be able to get a "real"
        // hash modulo signature, which we may not have
        let hash = tx.sighash(self.get_chainid().await?.as_u64());
        receipt.transaction_hash = hash;

        let mut pending = PendingTransaction::new(hash, self.provider());
        // Set the future to resolve immediately to the populated receipt when polled.
        // TODO: handle confirmations > 1. Likely need a dummy Provider that impls
        // get_block_number() using internal evm
        pending.set_state(PendingTxState::CheckingReceipt(Some(receipt)));
        Ok(pending)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        _block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        // Simulate an eth_call by saving the state, running the tx, then resetting state
        let state = (*self.vm().await.state()).clone();

        let res = self.apply_tx(tx).await?;
        let bytes = match res.output {
            TxOutput::CallRes(b) => b,
            // For a contract creation tx, return the deployed bytecode
            TxOutput::CreateRes(addr) => self.get_code(addr, None).await?,
        };

        self.vm_mut().await.reset(state);

        Ok(bytes)
    }

    // Copied from Provider::fill_transaction because we need other middleware
    // method calls to be captured by Forge
    async fn fill_transaction(
        &self,
        tx: &mut TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<(), Self::Error> {
        if let Some(default_sender) = self.default_sender() {
            if tx.from().is_none() {
                tx.set_from(default_sender);
            }
        }

        // TODO: Can we poll the futures below at the same time?
        // Access List + Name resolution and then Gas price + Gas

        // set the ENS name
        if let Some(NameOrAddress::Name(ref ens_name)) = tx.to() {
            let addr = self.resolve_name(ens_name).await?;
            tx.set_to(addr);
        }

        // estimate the gas without the access list
        let gas = maybe(tx.gas().cloned(), self.estimate_gas(tx)).await?;
        let mut al_used = false;

        // set the access lists
        if let Some(access_list) = tx.access_list() {
            if access_list.0.is_empty() {
                if let Ok(al_with_gas) = self.create_access_list(tx, block).await {
                    // only set the access list if the used gas is less than the
                    // normally estimated gas
                    if al_with_gas.gas_used < gas {
                        tx.set_access_list(al_with_gas.access_list);
                        tx.set_gas(al_with_gas.gas_used);
                        al_used = true;
                    }
                }
            }
        }

        if !al_used {
            tx.set_gas(gas);
        }

        match tx {
            TypedTransaction::Eip2930(_) | TypedTransaction::Legacy(_) => {
                let gas_price = maybe(tx.gas_price(), self.get_gas_price()).await?;
                tx.set_gas_price(gas_price);
            }
            TypedTransaction::Eip1559(ref mut inner) => {
                if inner.max_fee_per_gas.is_none() || inner.max_priority_fee_per_gas.is_none() {
                    let (max_fee_per_gas, max_priority_fee_per_gas) =
                        self.estimate_eip1559_fees(None).await?;
                    inner.max_fee_per_gas = Some(max_fee_per_gas);
                    inner.max_priority_fee_per_gas = Some(max_priority_fee_per_gas);
                };
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethers_core::types::{Address, TransactionRequest};
    use ethers_providers::Provider;
    use evm_adapters::sputnik::{
        helpers::{new_backend, CFG, GAS_LIMIT, VICINITY},
        Executor, PRECOMPILES_MAP,
    };

    #[derive(Debug, Clone, Copy)]
    pub struct NullProvider;
    impl NullProvider {
        pub fn new() -> Self {
            Self
        }
    }
    #[async_trait]
    impl JsonRpcClient for NullProvider {
        type Error = ProviderError;

        async fn request<T, R>(&self, _method: &str, _params: T) -> Result<R, Self::Error>
        where
            T: std::fmt::Debug + serde::Serialize + Send + Sync,
            R: serde::de::DeserializeOwned,
        {
            unreachable!("Cannot send requests")
        }
    }

    #[tokio::test]
    async fn test_forge() {
        let from: Address = "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8".parse().unwrap();
        let to: Address = "0xD3D13a578a53685B4ac36A1Bab31912D2B2A2F36".parse().unwrap();

        let provider = Provider::new(NullProvider);

        let backend = new_backend(&*VICINITY, Default::default());
        let vm = Executor::new(GAS_LIMIT, &*CFG, &backend, &*PRECOMPILES_MAP);
        let forge = Forge::new(provider, Arc::new(RwLock::new(vm)));

        let tx = TransactionRequest::new().to(to).from(from).value(1).gas(2300);
        let receipt = forge.send_transaction(tx, None).await.unwrap().await.unwrap();
        dbg!(receipt);
    }
}
