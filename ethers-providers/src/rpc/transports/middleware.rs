#![cfg(not(target_arch = "wasm32"))]
use crate::rpc::transports::http::ClientError;
use anyhow::anyhow;
use http::response::Builder;
use reqwest_chain::Chainer;
use reqwest_middleware::Error;
use tracing::trace;
use url::Url;

/// Middleware for switching between providers on failures
pub struct SwitchProviderMiddleware {
    /// Rpc providers to be used for retries of failed requests
    pub providers: Vec<Url>,
}

#[derive(Default, Debug)]
pub struct LocalState {
    pub active_provider_index: usize,
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
        let mut next_state = |client_error: ClientError| {
            let next_index = _state.active_provider_index + 1;
            if next_index >= self.providers.len() {
                trace!(target:"ethers-providers", "Providers have been exhausted");

                Err(anyhow!(client_error))?;
            }
            _state.active_provider_index = next_index;
            let next_provider = self.providers[next_index].clone();
            let url_ref = request.url_mut();

            *url_ref = next_provider;
            trace!(target:"ethers-providers", "Retrying request with new provider {url_ref:?}");
            Ok::<_, anyhow::Error>(())
        };

        match result {
            Ok(response) => {
                let body = response.bytes().await?;

                match serde_json::from_slice(&body) {
                    Ok(crate::rpc::common::Response::Success { result: _, .. }) => {
                        let http_response = Builder::new()
                            .status(200)
                            .body(body.clone())
                            .map_err(|err| Error::Middleware(anyhow!("Error {err:?}")))?;
                        return Ok(Some(reqwest::Response::from(http_response)));
                    }
                    Ok(crate::rpc::common::Response::Error { error, .. }) => {
                        let _ = next_state(ClientError::JsonRpcError(error))?;
                    }
                    Ok(_) => {
                        let err = ClientError::SerdeJson {
                            err: serde::de::Error::custom(
                                "unexpected notification over HTTP transport",
                            ),
                            text: String::from_utf8_lossy(&body).to_string(),
                        };
                        let _ = next_state(err)?;
                    }
                    Err(err) => {
                        let error = ClientError::SerdeJson {
                            err,
                            text: String::from_utf8_lossy(&body).to_string(),
                        };

                        let _ = next_state(error)?;
                    }
                };
            }
            Err(e) => {
                trace!(target:"ethers-providers", "Possibly encountered an os error submitting request, switching provider {e:?}");
                let _ = next_state(ClientError::MiddlewareError(e))?;
            }
        }

        Ok(None)
    }

    fn max_chain_length(&self) -> u32 {
        self.providers.len() as u32
    }
}

#[cfg(test)]
mod test {
    use crate::{Http, Middleware, Provider};
    use ethers_core::types::{Block, EIP1186ProofResponse, H160, H256};
    use reqwest::Url;

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_get_block_by_number() {
        let providers = vec![
            Url::parse("http://localhost:3500").unwrap(),
            Url::parse("https://www.noderpc.xyz/rpc-mainnet/public").unwrap(),
        ];

        let http_provider = Http::new_client_with_chain_middleware(providers, None);

        let block_num = "latest";
        let txn_details = false;
        let params = (block_num, txn_details);

        let provider = Provider::<Http>::new(http_provider.clone());

        let block: Block<H256> = provider.request("eth_getBlockByNumber", params).await.unwrap();
        assert!(block.hash.is_some())
    }

    #[tokio::test]
    async fn test_switch_provider_middleware_for_json_rpc_get_proof() {
        let providers = vec![
            Url::parse("http://localhost:3500").unwrap(),
            Url::parse("https://docs-demo.quiknode.pro").unwrap(),
        ];

        let http_provider = Http::new_client_with_chain_middleware(providers, None);

        let from =
            H160::from_slice(&hex::decode("7F0d15C7FAae65896648C8273B6d7E43f58Fa842").unwrap());
        let locations = vec![H256::from_slice(
            &hex::decode("56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421")
                .unwrap(),
        )];

        let provider = Provider::<Http>::new(http_provider.clone());

        let proof: EIP1186ProofResponse = provider.get_proof(from, locations, None).await.unwrap();

        assert_eq!(proof.address, from);
    }
}
