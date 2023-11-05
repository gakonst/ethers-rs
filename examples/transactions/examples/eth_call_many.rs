use ethers::{
    providers::{Http, Middleware, Provider},
    types::{
        Address, BlockId, Bytes, EthCallManyBalanceDiff, EthCallManyBundle, TransactionRequest,
        H160, U256,
    },
};
use eyre::Result;
use std::{collections::HashMap, str::FromStr};

/// use `debug_traceCall` to fetch traces
/// requires, a valid endpoint in `RPC_URL` env var that supports `debug_traceCall`
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let client = Provider::<Http>::try_from(url)?;
        let tx = TransactionRequest::new().from(Address::from_str("0xdeadbeef29292929192939494959594933929292").unwrap()).to(Address::from_str("0xde929f939d939d393f939393f93939f393929023").unwrap()).gas_price("0x5D21DBA00").gas("0x7a120").data(Bytes::from_str("0xf00d4b5d00000000000000000000000001291230982139282304923482304912923823920000000000000000000000001293123098123928310239129839291010293810").unwrap());
        let req = vec![EthCallManyBundle { transactions: vec![tx.clone()], block_override: None }];
        let mut hashmap: HashMap<H160, EthCallManyBalanceDiff> = HashMap::new();
        hashmap.insert(
            "0xDeaDbEEF29292929192939494959594933929292".parse::<H160>().unwrap(),
            EthCallManyBalanceDiff {
                balance: U256::from(
                    "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                ),
            },
        );
        let traces = client.eth_call_many(req, Some(hashmap), None).await?;
        println!("{traces:?}");
    }

    Ok(())
}
