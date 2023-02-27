use ethers_core::{
    types::{TransactionRequest, U256},
    utils::Anvil,
};
use ethers_providers::{Http, Middleware, Provider};
use std::convert::TryFrom;

#[tokio::test]
async fn txpool() {
    let anvil = Anvil::new().block_time(5u64).spawn();
    let provider = Provider::<Http>::try_from(anvil.endpoint()).unwrap();

    let account = provider.get_accounts().await.unwrap()[0];
    let value: u64 = 42;
    let gas_price = U256::from_dec_str("221435145689").unwrap();
    let tx = TransactionRequest::new().to(account).from(account).value(value).gas_price(gas_price);

    // send a few transactions
    for _ in 0..10 {
        drop(provider.send_transaction(tx.clone(), None).await.unwrap());
    }

    // we gave a 5s block time, should be plenty for us to get the txpool's content
    let status = provider.txpool_status().await.unwrap();
    assert_eq!(status.pending.as_u64(), 10);
    assert_eq!(status.queued.as_u64(), 0);

    let inspect = provider.txpool_inspect().await.unwrap();
    assert!(inspect.queued.is_empty());
    let summary = inspect.pending.get(&account).unwrap();
    for i in 0..10 {
        let tx_summary = summary.get(&i.to_string()).unwrap();
        assert_eq!(tx_summary.gas_price, gas_price);
        assert_eq!(tx_summary.value, value.into());
        assert_eq!(tx_summary.gas, 21000.into());
        assert_eq!(tx_summary.to.unwrap(), account);
    }

    let content = provider.txpool_content().await.unwrap();
    assert!(content.queued.is_empty());
    let content = content.pending.get(&account).unwrap();

    for nonce in 0..10 {
        assert!(content.contains_key(&nonce.to_string()));
    }
}
