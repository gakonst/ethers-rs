use async_trait::async_trait;
use ethers_core::types::{
    transaction::eip2718::TypedTransaction, Address, BlockId, Bytes, NameOrAddress,
    TransactionReceipt,
};
use ethers_providers::{
    JsonRpcClient, Middleware, PendingTransaction, PendingTxState, Provider, ProviderError,
};
use evm_adapters::Evm;
use std::{fmt::Debug, marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

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
    pub vm: Arc<Mutex<V>>,
    provider: Provider<NullProvider>,
    _state: PhantomData<S>,
}
impl<V, S> Forge<V, S> {
    pub fn new(vm: Arc<Mutex<V>>) -> Self {
        Self { vm, provider: Provider::new(NullProvider), _state: PhantomData }
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
    E: Evm<S> + Send + Sync,
    S: Send + Sync + Debug,
{
    type Error = ProviderError;
    type Provider = NullProvider;
    type Inner = Self;

    fn inner(&self) -> &Self::Inner {
        unreachable!("There is no inner provider here")
    }

    fn provider(&self) -> &Provider<Self::Provider> {
        &self.provider
    }

    async fn send_transaction<T: Into<TypedTransaction> + Send + Sync>(
        &self,
        tx: T,
        block: Option<BlockId>,
    ) -> Result<PendingTransaction<'_, Self::Provider>, Self::Error> {
        let mut tx = tx.into();
        // Will panic if gas or gas price aren't set, because we don't really have a provider
        self.provider().fill_transaction(&mut tx, block).await?;

        // Pull fields from tx to pass to evm
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

        // receipt to populate with the result of running the partial tx
        let mut receipt = TransactionReceipt::default();

        let mut lock = self.vm.lock().await;

        if *from == Address::zero() {
            // contract deployment
            let (addr, exit, gas, _) = lock.deploy(*from, data.clone(), *val).unwrap();
            receipt.gas_used = Some(gas.into());
            receipt.status = Some((if E::is_success(&exit) { 1usize } else { 0 }).into());
            receipt.contract_address = Some(addr);
        } else {
            // (contract) call
            let (_bytes, exit, gas, _) = lock.call_raw(*from, to, data, *val, false).unwrap();
            receipt.gas_used = Some(gas.into());
            receipt.status = Some((if E::is_success(&exit) { 1usize } else { 0 }).into());
        }

        // Fake the tx hash for the receipt. Should be able to get a "real"
        // hash modulo signature, which we may not have
        let hash = tx.sighash(1usize);
        receipt.transaction_hash = hash;
        // receipt.transaction_index = 0usize.into();
        // receipt.cumulative_gas_used = 0usize.into();

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
        let mut lock = self.vm.lock().await;
        let res = lock.call_raw(*from, to, data, *val, false).unwrap();
        Ok(res.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethers_core::types::TransactionRequest;
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
        let forge = Forge::new(Arc::new(Mutex::new(vm)));

        let tx = TransactionRequest::new().to(to).from(from).value(1).gas(2300).gas_price(1);
        let receipt = forge.send_transaction(tx, None).await.unwrap().await.unwrap();
        dbg!(receipt);
    }
}
