//! `send_eth infura` subcommand

#![allow(clippy::never_loop)]
use crate::{application::APP, prelude::*};
use abscissa_core::{Clap, Command, Runnable};
use ethers_core::types::{TransactionRequest, H160};

use ethers_middleware::SignerMiddleware;
use ethers_providers::{Http, Middleware, Provider};
use ethers_signers::{LocalWallet, Signer, Wallet};
use signatory::FsKeyStore;
use std::{convert::TryFrom, path};

/// `infura` subcommand
#[derive(Command, Debug, Default, Clap)]
pub struct RopstenCmd {
    pub args: Vec<String>,
}

impl Runnable for RopstenCmd {
    fn run(&self) {
        abscissa_tokio::run(&APP, async {
            let provider = self.args.get(0).expect("Infura url and key are required").to_owned();
            let provider = Provider::<Http>::try_from(provider)
                .expect("Could not connect to ganache endpoint");
            let keystore = path::Path::new("/tmp/keystore");
            let keystore = FsKeyStore::create_or_open(keystore).expect("Could not open keystore");
            let name = self.args.get(1).expect("wallet key is required").to_owned();
            let name = &name.parse().expect("Could not parse name");
            let key = keystore.load(name).expect("Could not load key");
            let key = key
                .to_pem()
                .parse::<k256::elliptic_curve::SecretKey<k256::Secp256k1>>()
                .expect("Could not parse key");
            let wallet: LocalWallet = Wallet::from(key);
            let wallet = wallet.with_chain_id(3u64);
            let client = SignerMiddleware::new(provider, wallet);
            // Get Receiver's account from args
            let to = self.args.get(2).expect("Receiver's account is required");
            // Parse Receiver's account to H160 from String
            let to = to.parse::<H160>().expect("could not parse recievers account");
            // Get ETH value from args
            let value = self.args.get(3).expect("Value is required");
            // Parse ETH value to i32
            let value = value.parse::<i32>().expect("could not parse value");
            // Craft transaction
            let tx = TransactionRequest::new().to(to).value(value);
            // broadcast it via the eth_sendTransaction API
            let tx = client.send_transaction(tx, None).await.unwrap().await.unwrap();
            println!("{}", serde_json::to_string(&tx).unwrap());
        })
        .unwrap_or_else(|e| {
            status_err!("executor exited with error: {}", e);
            std::process::exit(1);
        });
    }
}
