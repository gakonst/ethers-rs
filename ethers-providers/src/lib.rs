#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![doc = include_str!("../README.md")]

mod ext;
pub use ext::*;

mod rpc;
pub use rpc::*;

pub mod utils;
pub use utils::*;

/// Errors
pub mod errors;
pub use errors::{MiddlewareError, ProviderError, RpcError};

mod pending_transaction;
pub use pending_transaction::PendingTransaction;

mod pending_escalator;
pub use pending_escalator::EscalatingPending;

mod log_query;
pub use log_query::{LogQuery, LogQueryError};

mod stream;
pub use futures_util::StreamExt;
pub use stream::{
    interval, FilterWatcher, TransactionStream, DEFAULT_LOCAL_POLL_INTERVAL, DEFAULT_POLL_INTERVAL,
};

mod pubsub;
pub use pubsub::{PubsubClient, SubscriptionStream};

pub mod call_raw;

pub mod middleware;
pub use middleware::*;

#[allow(deprecated)]
pub use test_provider::{GOERLI, MAINNET, ROPSTEN, SEPOLIA};

/// Pre-instantiated Infura HTTP clients which rotate through multiple API keys
/// to prevent rate limits
pub mod test_provider {
    use super::*;
    use crate::Http;
    use once_cell::sync::Lazy;
    use std::{convert::TryFrom, iter::Cycle, slice::Iter, sync::Mutex};

    // List of infura keys to rotate through so we don't get rate limited
    const INFURA_KEYS: &[&str] = &["15e8aaed6f894d63a0f6a0206c006cdd"];

    pub static MAINNET: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "mainnet"));
    pub static GOERLI: Lazy<TestProvider> = Lazy::new(|| TestProvider::new(INFURA_KEYS, "goerli"));
    pub static SEPOLIA: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "sepolia"));

    #[deprecated = "Ropsten testnet has been deprecated in favor of Goerli or Sepolia."]
    pub static ROPSTEN: Lazy<TestProvider> =
        Lazy::new(|| TestProvider::new(INFURA_KEYS, "ropsten"));

    #[derive(Debug)]
    pub struct TestProvider {
        network: String,
        keys: Mutex<Cycle<Iter<'static, &'static str>>>,
    }

    impl TestProvider {
        pub fn new(keys: &'static [&'static str], network: impl Into<String>) -> Self {
            Self { keys: keys.iter().cycle().into(), network: network.into() }
        }

        pub fn url(&self) -> String {
            let Self { network, keys } = self;
            let key = keys.lock().unwrap().next().unwrap();
            format!("https://{network}.infura.io/v3/{key}")
        }

        pub fn provider(&self) -> Provider<Http> {
            Provider::try_from(self.url().as_str()).unwrap()
        }

        #[cfg(feature = "ws")]
        pub async fn ws(&self) -> Provider<crate::Ws> {
            let url = format!(
                "wss://{}.infura.io/ws/v3/{}",
                self.network,
                self.keys.lock().unwrap().next().unwrap()
            );
            Provider::connect(url.as_str()).await.unwrap()
        }
    }
}
