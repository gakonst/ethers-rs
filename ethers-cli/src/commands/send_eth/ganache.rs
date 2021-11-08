//! `send_eth ganache` subcommand

#![allow(clippy::never_loop)]
use crate::{application::APP, prelude::*};
use abscissa_core::{Clap, Command, Runnable};
use ethers_core::{
    types::{BlockNumber, TransactionRequest, H160},
    utils::Ganache,
};
use ethers_providers::{Http, Middleware, Provider};
use std::convert::TryFrom;

/// `ganache` subcommand
#[derive(Command, Debug, Default, Clap)]
pub struct GanacheCmd {
    pub args: Vec<String>,
}

impl Runnable for GanacheCmd {
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            // Spawn Ganache Instance
            let ganache = Ganache::new().port(8545u16)
            .mnemonic("abstract vacuum mammal awkward pudding scene penalty purchase dinner depart evoke puzzle")
            .spawn();
            let provider = Provider::<Http>::try_from(ganache.endpoint())
                .expect("Could not connect to ganache endpoint");
            let accounts = provider.get_accounts().await.unwrap();
            println!("Accounts:{:?}", accounts);
            // Get Sender's account from args
            let from = self.args.get(0).expect("Sender's account is required");
            // Parse Sender's account to H160 from String
            let from = from.parse::<H160>().expect("could not parse senders account");
            // Get Receiver's account from args
            let to = self.args.get(1).expect("Receiver's account is required");
            // Parse Receiver's account to H160 from String
            let to = to.parse::<H160>().expect("could not parse recievers account");
            // Get ETH value from args
            let value = self.args.get(2).expect("Value is required");
            // Parse ETH value to i32
            let value = value.parse::<i32>().expect("could not parse value");
            // Craft transaction
            let tx = TransactionRequest::new().to(to).value(value).from(from); // specify the `from` field so that the client knows which account to use
            let balance_before = provider.get_balance(from, None).await.unwrap();
            // broadcast it via the eth_sendTransaction API
            let tx = provider.send_transaction(tx, None).await.unwrap().await.unwrap();
            println!("{}", serde_json::to_string(&tx).unwrap());

            let nonce1 = provider
                .get_transaction_count(from, Some(BlockNumber::Latest.into()))
                .await
                .unwrap();

            let nonce2 = provider
                .get_transaction_count(from, Some(BlockNumber::Number(0.into()).into()))
                .await
                .unwrap();

            assert!(nonce2 < nonce1);

            let balance_after = provider.get_balance(from, None).await.unwrap();
            assert!(balance_after < balance_before);

            println!("Balance before {}", balance_before);
            println!("Balance after {}", balance_after);
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}
