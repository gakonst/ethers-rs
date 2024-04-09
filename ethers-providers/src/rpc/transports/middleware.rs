use std::collections::HashMap;

use anyhow::anyhow;
use reqwest::{Response, StatusCode, Url};
use reqwest_chain::Chainer;
use reqwest_middleware::Error;
use ethers_core::types::transaction::request;
use crate::HttpClientError::ReqwestError;
use crate::ProviderError;
use crate::rpc::transports::http::{ClientError, Provider};

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
                let res = _state.prev_stat.iter().filter_map(|(_, error_option)| {
                    error_option.as_ref().and_then(|error| match error {
                        ReqwestError(err) if err.status() == Some(StatusCode::NOT_FOUND) => Some(()),
                        _ => None
                    })
                }).any(|_| true);

                if res {
                    return Err(anyhow!("All providers returned {:?}", StatusCode::NOT_FOUND))?
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
            Ok(response) => {
                if response.status() == StatusCode::OK {
                    return Ok(Some(response));
                };

                match response.error_for_status() {
                    Ok(_) => {}
                    Err(err) => {
                        let _ = next_state(Some(ReqwestError(err)))?;
                    }
                }
            },
            Err(e) => {
                log::trace!(target:"ethers-providers", "Possibly encountered an os error submitting request, switching provider {e:?}");
                let _ = next_state(None)?;
            },
        }

        Ok(None)
    }

    fn max_chain_length(&self) -> u32 {
        u32::MAX
    }
}
