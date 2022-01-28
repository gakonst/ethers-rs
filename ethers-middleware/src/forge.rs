use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, NameOrAddress,
    TransactionReceipt, U256, U64,
};
use ethers_providers::{
    maybe, JsonRpcClient, Middleware, PendingTransaction, PendingTxState, Provider, ProviderError,
};
use evm_adapters::{
    sputnik::{Executor, SputnikExecutor},
    Evm,
};
use sputnik::backend::Backend;
use std::{
    fmt::Debug,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use tokio::sync::RwLock;

pub trait VmShow {
    fn gas_price(&self) -> U256;
    fn block_number(&self) -> U256;
    fn chain_id(&self) -> U256;
    fn balance(&self, from: Address, block: Option<BlockId>) -> U256;
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
    // TODO: incorporate block parameter
    fn balance(&self, addr: Address, block: Option<BlockId>) -> U256 {
        self.executor.state().basic(addr).balance
    }
}

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

#[derive(Clone)]
pub struct Forge<V, S> {
    pub vm: Arc<RwLock<V>>,
    provider: Provider<NullProvider>,
    _state: PhantomData<S>,
}
impl<V, S> Forge<V, S> {
    pub fn new(vm: Arc<RwLock<V>>) -> Self {
        Self { vm, provider: Provider::new(NullProvider), _state: PhantomData }
    }
    // async fn vm(&self) -> tokio::sync::RwLockReadGuard<'_, V> {
    async fn vm(&self) -> impl Deref<Target = V> + '_ {
        self.vm.read().await
    }
    async fn vm_mut(&self) -> impl DerefMut<Target = V> + '_ {
        self.vm.write().await
    }
}
// Stand-in impl because some sputnik component is not Debug
impl<V, S> Debug for Forge<V, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forge").finish()
    }
}

#[async_trait]
impl<E, S> Middleware for Forge<E, S>
where
    E: Evm<S> + VmShow + Send + Sync,
    S: Send + Sync + Debug,
{
    type Error = ProviderError;
    type Provider = NullProvider;
    type Inner = Provider<NullProvider>;

    fn inner(&self) -> &Self::Inner {
        self.provider()
    }

    fn provider(&self) -> &Provider<Self::Provider> {
        &self.provider
    }

    async fn estimate_gas(&self, _tx: &TypedTransaction) -> Result<U256, Self::Error> {
        Ok(0usize.into())
    }

    async fn get_gas_price(&self) -> Result<U256, Self::Error> {
        Ok(self.vm().await.gas_price())
    }

    async fn get_block_number(&self) -> Result<U64, Self::Error> {
        Ok(self.vm().await.block_number().as_u64().into())
    }

    async fn get_chainid(&self) -> Result<U256, Self::Error> {
        Ok(self.vm().await.chain_id())
    }

    async fn get_balance<T: Into<NameOrAddress> + Send + Sync>(
        &self,
        from: T,
        block: Option<BlockId>,
    ) -> Result<U256, Self::Error> {
        let addr = match from.into() {
            NameOrAddress::Name(ref ens) => self.resolve_name(ens).await?,
            NameOrAddress::Address(a) => a,
        };
        Ok(self.vm().await.balance(addr, block))
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

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx = tx.into();
        // Will panic if gas or gas price aren't set, because we don't really have a provider
        self.fill_transaction(&mut tx, block).await?;

        // Pull fields from tx to pass to evm
        let from = tx.from().unwrap();
        let maybe_to = tx.to().map(|id| async move {
            match id {
                NameOrAddress::Name(ens) => self.resolve_name(ens).await.unwrap(),
                NameOrAddress::Address(addr) => *addr,
            }
        });
        let data = tx.data().map_or(Default::default(), |d| d.clone());
        let val = tx.value().unwrap();

        // receipt to populate with the result of running the partial tx
        let mut receipt = TransactionReceipt::default();

        // let mut lock = self.vm.write().await;

        if let Some(fut) = maybe_to {
            let to = fut.await;
            // (contract) call
            let (_bytes, exit, gas, _) =
                self.vm_mut().await.call_raw(*from, to, data, *val, false).unwrap();
            receipt.gas_used = Some(gas.into());
            receipt.status = Some((if E::is_success(&exit) { 1usize } else { 0 }).into());
        } else {
            // contract deployment
            let (addr, exit, gas, _) =
                self.vm_mut().await.deploy(*from, data.clone(), *val).unwrap();
            receipt.gas_used = Some(gas.into());
            receipt.status = Some((if E::is_success(&exit) { 1usize } else { 0 }).into());
            receipt.contract_address = Some(addr);
        }

        // Fake the tx hash for the receipt. Should be able to get a "real"
        // hash modulo signature, which we may not have
        let hash = tx.sighash(1usize);
        receipt.transaction_hash = hash;

        let mut pending = PendingTransaction::new(hash, self.provider());
        // Set the future to resolve immediately to the populated receipt when polled.
        // TODO: handle confirmations > 1
        pending.set_state(PendingTxState::CheckingReceipt(Some(receipt)));
        Ok(pending)
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        _block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        let from = tx.from().unwrap();
        let to = match tx.to().unwrap() {
            NameOrAddress::Name(ens) => self.resolve_name(ens).await?,
            NameOrAddress::Address(addr) => *addr,
        };
        let data = match tx.data() {
            Some(data) => data.clone(),
            _ => Default::default(),
        };
        let val = tx.value().unwrap();
        let mut lock = self.vm.write().await;
        let res = lock.call_raw(*from, to, data, *val, false).unwrap();
        Ok(res.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethers_core::types::{Address, TransactionRequest};
    use evm_adapters::sputnik::{
        helpers::{new_backend, CFG, GAS_LIMIT, VICINITY},
        Executor, PRECOMPILES_MAP,
    };

    #[tokio::test]
    async fn test_forge() {
        let from: Address = "0xEA674fdDe714fd979de3EdF0F56AA9716B898ec8".parse().unwrap();
        let to: Address = "0xD3D13a578a53685B4ac36A1Bab31912D2B2A2F36".parse().unwrap();

        let backend = new_backend(&*VICINITY, Default::default());
        let vm = Executor::new(GAS_LIMIT, &*CFG, &backend, &*PRECOMPILES_MAP);
        let forge = Forge::new(Arc::new(RwLock::new(vm)));

        let tx = TransactionRequest::new().to(to).from(from).value(1).gas(2300).gas_price(1);
        let receipt = forge.send_transaction(tx, None).await.unwrap().await.unwrap();
        dbg!(receipt);
    }
}
