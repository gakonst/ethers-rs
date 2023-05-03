use async_trait::async_trait;
use ethers_contract::{EthAbiCodec, EthAbiType, EthError};
use ethers_core::{
    abi::AbiEncode,
    types::{transaction::eip2718::TypedTransaction, *},
};
use ethers_providers::{Middleware, MiddlewareError};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug)]
/// Middleware used for doing offchain data retrieval
/// See https://eips.ethereum.org/EIPS/eip-3668
pub struct CCIPReadMiddleware<M> {
    inner: M,
}

#[derive(Debug, Clone, EthError)]
#[etherror(
    name = "OffchainLookup",
    abi = "OffchainLookup(address, string[], bytes, bytes4, bytes)"
)]
struct OffchainLookup {
    sender: Address,
    urls: Vec<String>,
    call_data: Bytes,
    callback_function: [u8; 4], // Bytes4
    extra_data: Bytes,
}

#[derive(Deserialize)]
struct CCIPReadResult {
    data: Bytes,
}

#[derive(Clone, Debug, Default, PartialEq, EthAbiType, EthAbiCodec)]
struct Callback {
    call_data: Bytes,
    extra_data: Bytes,
}

impl OffchainLookup {
    fn templatize(&self, url: &String) -> String {
        url.replace("{data}", &self.call_data.to_string())
            .replace("{sender}", &self.sender.to_string())
    }

    async fn fetch(&self) -> Option<CCIPReadResult> {
        // 6. Construct a request URL by replacing sender with the lowercase 0x-prefixed hexadecimal
        // formatted sender parameter, and replacing data with the the 0x-prefixed hexadecimal
        // formatted callData parameter. The client may choose which URLs to try in which order, but
        // SHOULD prioritise URLs earlier in the list over those later in the list.
        let mut urls = self.urls.iter().map(|url| self.templatize(url));
        // TODO: try all urls
        let url = urls.next().unwrap();

        // 7. Make an HTTP GET request to the request URL.
        let response = reqwest::get(&url).await;

        match response {
            Ok(response) => {
                match response.status().as_u16() {
                    // 8. If the response code from step (5) is in the range 400-499, return an
                    // error to the caller and stop.
                    400..=499 => todo!(),
                    // 9. If the response code from step (5) is in the range 500-599, go back to
                    // step (5) and pick a different URL, or stop if there are no further URLs to
                    // try.
                    500..=599 => todo!(),
                    _ => {
                        let result = response.json::<CCIPReadResult>().await;
                        match result {
                            Ok(result) => Some(result),
                            Err(e) => {
                                eprintln!("Error parsing response: {:?}", e);
                                return None
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching data: {:?}", e);
                return None
            }
        }
    }

    fn encode_callback_data(&self, result: CCIPReadResult) -> Bytes {
        let callback = Callback { call_data: result.data, extra_data: self.extra_data.clone() };

        return Bytes::from([self.callback_function.to_vec(), callback.encode()].concat())
    }
}

impl<M> CCIPReadMiddleware<M>
where
    M: Middleware,
{
    pub fn new(inner: M) -> Self {
        Self { inner }
    }
}

#[derive(Error, Debug)]
/// Thrown when an error happens at the CCIP Read
pub enum CCIPReadError<M: Middleware> {
    /// Thrown when the internal middleware errors
    #[error("{0}")]
    MiddlewareError(M::Error),
}

impl<M: Middleware> MiddlewareError for CCIPReadError<M> {
    type Inner = M::Error;

    fn from_err(src: M::Error) -> Self {
        CCIPReadError::MiddlewareError(src)
    }

    fn as_inner(&self) -> Option<&Self::Inner> {
        match self {
            CCIPReadError::MiddlewareError(e) => Some(e),
        }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<M> Middleware for CCIPReadMiddleware<M>
where
    M: Middleware,
{
    type Error = CCIPReadError<M>;
    type Provider = M::Provider;
    type Inner = M;

    fn inner(&self) -> &M {
        &self.inner
    }

    async fn call(
        &self,
        tx: &TypedTransaction,
        block: Option<BlockId>,
    ) -> Result<Bytes, Self::Error> {
        // 1. Set data to the call data to supply to the contract, and to to the address of the
        // contract to call.
        let call_result = self.inner.call(tx, block).await;

        match call_result {
            // 2. Call the contract at address to function normally, supplying data as the input
            // data. If the function returns a successful result, return it to the caller and stop.
            Ok(bytes) => Ok(bytes),

            Err(e) => {
                match e.as_error_response().and_then(|e| OffchainLookup::from_rpc_response(e)) {
                    // 3. If the function returns an error other than OffchainLookup, return it to
                    // the caller in the usual fashion.
                    None => return Err(CCIPReadError::MiddlewareError(e)),

                    // 4. Otherwise, decode the sender, urls, callData, callbackFunction and
                    // extraData arguments from the OffchainLookup error.
                    Some(lookup) => {
                        // 5. If the sender field does not match the address of the contract that
                        // was called, return an error to the caller and stop.
                        if &lookup.sender != tx.to().unwrap().as_address().unwrap() {
                            return Err(CCIPReadError::MiddlewareError(e))
                        }

                        // see fetch for steps 6 - 9
                        let lookup_result = lookup.fetch().await;
                        match lookup_result {
                            // 10. Otherwise, replace data with an ABI-encoded call to the contract
                            // function specified by the 4-byte selector callbackFunction, supplying
                            // the data returned from step (7) and extraData from step (4), and
                            // return to step (1).
                            Some(lookup_result) => {
                                let mut new_tx = tx.clone();
                                new_tx.set_data(lookup.encode_callback_data(lookup_result));
                                return self.call(&new_tx, block).await
                            }
                            None => return Err(CCIPReadError::MiddlewareError(e)),
                        }
                    }
                }
            }
        }
    }
}
