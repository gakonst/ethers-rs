use crate::ProviderError;
use dashmap::DashMap;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

#[derive(Clone, Debug, Default)]
/// Simple in-memory K-V cache using concurrent dashmap which flushes
/// its state to disk on `Drop`.
pub struct Cache {
    path: PathBuf,
    // serialized request / response pair
    requests: DashMap<String, String>,
}

// Helper type for (de)serialization
#[derive(Serialize, Deserialize)]
struct CachedRequest<'a, T> {
    method: &'a str,
    params: T,
}

impl Cache {
    /// Instantiates a new cache at a file path.
    pub fn new(path: PathBuf) -> Result<Self, ProviderError> {
        // try to read the already existing requests
        let reader =
            BufReader::new(File::options().write(true).read(true).create(true).open(&path)?);
        let requests = serde_json::from_reader(reader).unwrap_or_default();
        Ok(Self { path, requests })
    }

    pub fn get<T: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: &T,
    ) -> Result<Option<R>, ProviderError> {
        let key = serde_json::to_string(&CachedRequest { method, params })?;
        let value = self.requests.get(&key);
        value.map(|x| serde_json::from_str(&x).map_err(ProviderError::SerdeJson)).transpose()
    }

    pub fn set<T: Serialize, R: Serialize>(
        &self,
        method: &str,
        params: T,
        response: R,
    ) -> Result<(), ProviderError> {
        let key = serde_json::to_string(&CachedRequest { method, params })?;
        let value = serde_json::to_string(&response)?;
        self.requests.insert(key, value);
        Ok(())
    }
}

impl Drop for Cache {
    fn drop(&mut self) {
        let file = match File::options().write(true).read(true).create(true).open(&self.path) {
            Ok(inner) => BufWriter::new(inner),
            Err(err) => {
                tracing::error!("could not open cache file {}", err);
                return
            }
        };

        // overwrite the cache
        if let Err(err) = serde_json::to_writer(file, &self.requests) {
            tracing::error!("could not write to cache file {}", err);
        };
    }
}

#[cfg(test)]
mod tests {
    use crate::{Middleware, Provider};
    use ethers_core::types::{Address, U256};

    #[tokio::test]
    async fn test_cache() {
        let tmp = tempfile::tempdir().unwrap();
        let cache = tmp.path().join("cache");
        let (provider, mock) = Provider::mocked();
        let provider = provider.with_cache(cache.clone());
        let addr = Address::random();

        assert!(provider.cache().unwrap().requests.is_empty());

        mock.push(U256::from(100u64)).unwrap();
        let res = provider.get_balance(addr, None).await.unwrap();
        assert_eq!(res, 100.into());

        assert!(!provider.cache().unwrap().requests.is_empty());
        dbg!(&provider.cache());
    }
}
