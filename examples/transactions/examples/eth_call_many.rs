use ethers::{
    providers::{Http, Middleware, Provider},
    types::{
        transaction::eip2718::TypedTransaction, Address, BlockNumber, Bytes,
        EthCallManyBalanceDiff, EthCallManyBundle, EthCallManyStateContext, TransactionRequest,
        H160, U256,
    },
};
use eyre::Result;
use std::{collections::HashMap, str::FromStr};

/// use `eth_callMany` to simulate output
/// requires, a valid endpoint in `RPC_URL` env var that supports `eth_callMany` (Erigon)
/// Example 1: of approving SHIBA INU (token) with Uniswap V2 as spender returns bool
/// Expected output: 0x0000000000000000000000000000000000000000000000000000000000000001
/// Example 2: transferring tokens while not having enough balance
/// Expected output: Empty error object
#[tokio::main]
async fn main() -> Result<()> {
    if let Ok(url) = std::env::var("RPC_URL") {
        let client = Provider::<Http>::try_from(url)?;
        let block_number = client.get_block_number().await.unwrap();
        let gas_fees = client.get_gas_price().await.unwrap();
        {
            let tx = TransactionRequest::new().from(Address::from_str("0xdeadbeef29292929192939494959594933929292").unwrap()).to(Address::from_str("0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce").unwrap()).gas_price(gas_fees).gas("0x7a120").data(Bytes::from_str("0x095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488d0000000000000000000000000000000000000000b3827a7d5189a7b76dac0000").unwrap());
            let req = vec![EthCallManyBundle {
                transactions: vec![TypedTransaction::Legacy(tx)],
                block_override: None,
            }];
            let mut hashmap: HashMap<H160, Option<EthCallManyBalanceDiff>> = HashMap::new();
            hashmap.insert(
                "0xDeaDbEEF29292929192939494959594933929292".parse::<H160>().unwrap(),
                Some(EthCallManyBalanceDiff {
                    balance: U256::from(
                        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                    ),
                }),
            );

            let state_context = EthCallManyStateContext {
                block_number: BlockNumber::Number(block_number),
                transaction_index: None,
            };
            let traces = client.eth_call_many(req, state_context, Some(hashmap)).await?;
            println!("{traces:?}");
        }

        {
            let tx = TransactionRequest::new().from(Address::from_str("0xdeadbeef29292929192939494959594933929292").unwrap()).to(Address::from_str("0x95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce").unwrap()).gas_price(gas_fees).gas("0x7a120").data(Bytes::from_str("0xa9059cbb000000000000000000000000deadbeef292929291929394949595949339292920000000000000000000000000000000000000000000000000000000000000005").unwrap());
            let req = vec![EthCallManyBundle {
                transactions: vec![TypedTransaction::Legacy(tx)],
                block_override: None,
            }];
            let mut hashmap: HashMap<H160, Option<EthCallManyBalanceDiff>> = HashMap::new();
            hashmap.insert(
                "0xDeaDbEEF29292929192939494959594933929292".parse::<H160>().unwrap(),
                Some(EthCallManyBalanceDiff {
                    balance: U256::from(
                        "0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF",
                    ),
                }),
            );

            let state_context = EthCallManyStateContext {
                block_number: BlockNumber::Number(block_number),
                transaction_index: None,
            };
            let traces = client.eth_call_many(req, state_context, Some(hashmap)).await?;
            println!("{traces:?}");
        }
    }

    Ok(())
}
