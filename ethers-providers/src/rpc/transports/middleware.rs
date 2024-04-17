use bytes::Bytes;
use std::collections::HashMap;

use crate::{
    rpc::transports::http::{ClientError, Provider},
    HttpClientError::{MiddlewareError, ReqwestError},
};
use anyhow::anyhow;
use http::response::Builder;
use reqwest::{Body, Response, StatusCode};
use reqwest_chain::Chainer;
use reqwest_middleware::Error;
use url::Url;

/// Middleware for switching between providers on failures
pub struct SwitchProviderMiddleware {
    /// Providers for the url
    pub providers: Vec<Url>,
}

#[derive(Default, Debug)]
pub struct LocalState {
    pub active_provider_index: usize,
    pub prev_stat: HashMap<usize, Option<ClientError>>,
}

impl SwitchProviderMiddleware {
    pub fn _new(providers: Vec<Url>) -> Self {
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
            let next_index = _state.active_provider_index + 1;
            if next_index >= self.providers.len() {
                return Err(anyhow!("Exiting Middleware"))?;
            }
            _state.active_provider_index = next_index;
            let next_provider = self.providers[next_index].clone();
            let url_ref = request.url_mut();

            *url_ref = next_provider.clone();
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
                            return Ok(None);
                        }
                    }
                };

                let mut body_vec = Vec::new();
                while let Some(chunk) = response.chunk().await? {
                    body_vec.extend_from_slice(&chunk);
                }

                let body = Bytes::from(body_vec);

                match serde_json::from_slice(&body) {
                    Ok(crate::rpc::common::Response::Success { result: _, .. }) => {
                        let http_response = Builder::new()
                            .status(200)
                            .body(body.clone())
                            .map_err(|err| Error::Middleware(anyhow!("Error {err:?}")))?;
                        return Ok(Some(reqwest::Response::from(http_response)));
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
    use crate::{Http, Provider};
    use ethers_core::types::{Block, EIP1186ProofResponse, H160, H256};
    use reqwest::Url;

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_get_block_by_number() {
        let providers = vec![
            Url::parse("http://localhost:3500").unwrap(),
            Url::parse("https://www.noderpc.xyz/rpc-mainnet/public").unwrap(),
        ];

        let http_provider = Http::new_client_with_chain_middleware(providers);

        let block_num = "0x12c1bc8";
        let txn_details = false;
        let params = (block_num, txn_details);

        let provider = Provider::<Http>::new(http_provider.clone());

        let block: Block<H256> = provider.request("eth_getBlockByNumber", params).await.unwrap();

        let expected_block_number: u64 = 19667912;
        assert_eq!(block.number.unwrap(), expected_block_number.into());
    }

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_rpc_get_proof() {
        let providers = vec![
            Url::parse("http://localhost:3500").unwrap(),
            Url::parse("https://docs-demo.quiknode.pro").unwrap(),
        ];

        let http_provider = Http::new_client_with_chain_middleware(providers);

        let address = "0x7F0d15C7FAae65896648C8273B6d7E43f58Fa842";
        let storage_key =
            vec!["0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421"];
        let block = "0x12c1bc8";

        let params = (address, storage_key, block);

        let provider = Provider::<Http>::new(http_provider.clone());

        let proof: EIP1186ProofResponse = provider.request("eth_getProof", params).await.unwrap();

        let expected_address = H160::from_slice(
            hex::decode("7f0d15c7faae65896648c8273b6d7e43f58fa842").unwrap().as_slice(),
        );

        assert_eq!(proof.address, expected_address);
    }
}
