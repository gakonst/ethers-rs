use bytes::Bytes;
use std::{collections::HashMap, io::Read};

use crate::{
    rpc::transports::http::{ClientError, Provider},
    HttpClientError::ReqwestError,
    JsonRpcClient, ProviderError,
};
use anyhow::anyhow;
use ethers_core::types::{transaction::request, Block, H256};
use reqwest::{Client, Response, StatusCode, Url};
use reqwest_chain::{ChainMiddleware, Chainer};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, Error};

/// Middleware for switching between providers on failures
pub struct SwitchProviderMiddleware {
    /// Providers for the url
    pub providers: Vec<Provider>,
}

#[derive(Default, Debug)]
pub struct LocalState {
    pub active_provider_index: usize,
    pub prev_stat: HashMap<usize, Option<ClientError>>,
}

impl SwitchProviderMiddleware {
    pub fn _new(providers: Vec<Provider>) -> Self {
        Self { providers }
    }
}

#[async_trait::async_trait]
impl Chainer for SwitchProviderMiddleware {
    type State = LocalState;

    async fn chain(
        &self,
        result: Result<reqwest::Response, Error>,
        _state: &mut Self::State,
        request: &mut reqwest::Request,
    ) -> Result<Option<reqwest::Response>, Error> {
        let mut next_state = |client_error: Option<ClientError>| {
            let active_index = _state.active_provider_index;
            _state.prev_stat.insert(active_index, client_error);
            let mut next_index = _state.active_provider_index + 1;
            if next_index >= self.providers.len() {
                let res = _state
                    .prev_stat
                    .iter()
                    .filter_map(|(_, error_option)| {
                        error_option.as_ref().and_then(|error| match error {
                            ReqwestError(err) if err.status() == Some(StatusCode::NOT_FOUND) => {
                                Some(())
                            }
                            _ => None,
                        })
                    })
                    .any(|_| true);

                if res {
                    return Err(anyhow!("All providers returned {:?}", StatusCode::NOT_FOUND))?;
                }
                next_index = 0;
            }
            _state.active_provider_index = next_index;
            let next_provider = self.providers[next_index].clone();
            let url_ref = request.url_mut();
            let new_url = next_provider.url();
            *url_ref = new_url.clone();
            log::trace!(target:"ethers-providers", "Retrying request with new provider {next_provider:?}");
            Ok::<_, anyhow::Error>(())
        };

        match result {
            Ok(mut response) => {
                if response.status() != StatusCode::OK {
                    match response.error_for_status_ref() {
                        Ok(_res) => (),
                        Err(err) => {
                            let _ = next_state(Some(ReqwestError(err)))?;
                            return Ok(None)
                        }
                    }
                };
                let mut body_vec = Vec::new();
                while let Some(chunk) = response.chunk().await? {
                    body_vec.extend_from_slice(&chunk);
                }

                let body = Bytes::from(body_vec);

                if !String::from_utf8_lossy(body.as_ref()).contains("jsonrpc") {
                    return Ok(Some(response));
                }

                match serde_json::from_slice(&body) {
                    Ok(crate::rpc::common::Response::Success { result, .. }) => {
                        return Ok(Some(response));
                    }
                    Ok(crate::rpc::common::Response::Error { error, .. }) => {
                        let _ = next_state(Some(ClientError::JsonRpcError(error)))?;
                    }
                    Ok(_) => {
                        let err = ClientError::SerdeJson {
                            err: serde::de::Error::custom(
                                "unexpected notification over HTTP transport",
                            ),
                            text: String::from_utf8_lossy(&body).to_string(),
                        };
                        let _ = next_state(Some(err))?;
                    }
                    Err(err) => {
                        let error = ClientError::SerdeJson {
                            err,
                            text: String::from_utf8_lossy(&body).to_string(),
                        };

                        let _ = next_state(Some(error))?;
                    }
                };
            }
            Err(e) => {
                log::trace!(target:"ethers-providers", "Possibly encountered an os error submitting request, switching provider {e:?}");
                let _ = next_state(None)?;
            }
        }

        Ok(None)
    }

    fn max_chain_length(&self) -> u32 {
        let provider_len = self.providers.len() as u32;
        provider_len + 1
    }
}

#[cfg(test)]
mod test {
    use crate::rpc::{
        common::Request,
        transports::{http::Provider, middleware::SwitchProviderMiddleware},
    };
    use reqwest::{Client, Url};
    use reqwest_chain::ChainMiddleware;
    use reqwest_middleware::ClientBuilder;

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_rpc_call() {
        let providers = vec![
            Provider::new(Url::parse("http://localhost:3500").unwrap()),
            Provider::new(Url::parse("https://www.noderpc.xyz/rpc-mainnet/public").unwrap()),
        ];

        let client = ClientBuilder::new(Client::new())
            .with(ChainMiddleware::new(SwitchProviderMiddleware::_new(providers.clone())))
            .build();

        let block_num = 100;
        let txn_details = false;
        let params = (block_num, txn_details);

        let payload = Request::new(0, "eth_getBlockByNumber", params);

        let res = client.post("https://eth.llamarpc.com").json(&payload).send().await.unwrap();

        assert!(res.status() == 200);
    }

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_http_call() {
        let providers = vec![
            Provider::new(Url::parse("http://localhost:3500").unwrap()),
            Provider::new(
                Url::parse(
                    "https://docs-demo.quiknode.pro/eth/v1/beacon/states/head/finality_checkpoints",
                )
                .unwrap(),
            ),
        ];

        let client = ClientBuilder::new(Client::new())
            .with(ChainMiddleware::new(SwitchProviderMiddleware::_new(providers.clone())))
            .build();

        let res = client.get("http://localhost:3500").send().await.unwrap();

        assert!(res.status() == 200);
    }
}
