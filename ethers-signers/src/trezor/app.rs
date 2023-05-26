#![allow(unused)]
use trezor_client::client::{AccessListItem as Trezor_AccessListItem, Trezor};

use futures_executor::block_on;
use futures_util::lock::Mutex;

use ethers_core::{
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, NameOrAddress, Signature, Transaction, TransactionRequest, TxHash, H256, U256,
    },
    utils::keccak256,
};
use home;
use std::{
    convert::TryFrom,
    env, fs,
    io::{Read, Write},
    path,
    path::PathBuf,
};
use thiserror::Error;

use super::types::*;

/// A Trezor Ethereum App.
///
/// This is a simple wrapper around the [Trezor transport](Trezor)
#[derive(Debug)]
pub struct TrezorEthereum {
    derivation: DerivationType,
    session_id: Vec<u8>,
    cache_dir: PathBuf,
    pub(crate) chain_id: u64,
    pub(crate) address: Address,
}

// we need firmware that supports EIP-1559 and EIP-712
const FIRMWARE_1_MIN_VERSION: &str = ">=1.11.1";
const FIRMWARE_2_MIN_VERSION: &str = ">=2.5.1";

// https://docs.trezor.io/trezor-firmware/common/communication/sessions.html
const SESSION_ID_LENGTH: usize = 32;
const SESSION_FILE_NAME: &str = "trezor.session";

impl TrezorEthereum {
    pub async fn new(
        derivation: DerivationType,
        chain_id: u64,
        cache_dir: Option<PathBuf>,
    ) -> Result<Self, TrezorError> {
        let cache_dir = (match cache_dir.or_else(home::home_dir) {
            Some(path) => path,
            None => match env::current_dir() {
                Ok(path) => path,
                Err(e) => return Err(TrezorError::CacheError(e.to_string())),
            },
        })
        .join(".ethers-rs")
        .join("trezor")
        .join("cache");

        let mut blank = Self {
            derivation: derivation.clone(),
            chain_id,
            cache_dir,
            address: Address::from([0_u8; 20]),
            session_id: vec![],
        };

        // Check if reachable
        blank.initate_session()?;
        blank.address = blank.get_address_with_path(&derivation).await?;
        Ok(blank)
    }

    fn check_version(version: String) -> Result<(), TrezorError> {
        let version = semver::Version::parse(&version)?;

        let min_version = match version.major {
            1 => FIRMWARE_1_MIN_VERSION,
            2 => FIRMWARE_2_MIN_VERSION,
            // unknown major version, possibly newer models that we don't know about yet
            // it's probably safe to assume they support EIP-1559 and EIP-712
            _ => return Ok(()),
        };

        let req = semver::VersionReq::parse(min_version)?;
        // Enforce firmware version is greater than "min_version"
        if !req.matches(&version) {
            return Err(TrezorError::UnsupportedFirmwareVersion(min_version.to_string()))
        }

        Ok(())
    }

    fn get_cached_session(&self) -> Result<Option<Vec<u8>>, TrezorError> {
        let mut session = [0; SESSION_ID_LENGTH];

        if let Ok(mut file) = fs::File::open(self.cache_dir.join(SESSION_FILE_NAME)) {
            file.read_exact(&mut session).map_err(|e| TrezorError::CacheError(e.to_string()))?;
            Ok(Some(session.to_vec()))
        } else {
            Ok(None)
        }
    }

    fn save_session(&mut self, session_id: Vec<u8>) -> Result<(), TrezorError> {
        fs::create_dir_all(&self.cache_dir).map_err(|e| TrezorError::CacheError(e.to_string()))?;

        let mut file = fs::File::create(self.cache_dir.join(SESSION_FILE_NAME))
            .map_err(|e| TrezorError::CacheError(e.to_string()))?;

        file.write_all(&session_id).map_err(|e| TrezorError::CacheError(e.to_string()))?;

        self.session_id = session_id;
        Ok(())
    }

    fn initate_session(&mut self) -> Result<(), TrezorError> {
        let mut client = trezor_client::unique(false)?;
        client.init_device(self.get_cached_session()?)?;

        let features = client.features().ok_or(TrezorError::FeaturesError)?;

        Self::check_version(format!(
            "{}.{}.{}",
            features.major_version(),
            features.minor_version(),
            features.patch_version()
        ))?;

        self.save_session(features.session_id().to_vec())?;

        Ok(())
    }

    /// You need to drop(client) once you're done with it
    fn get_client(&self, session_id: Vec<u8>) -> Result<Trezor, TrezorError> {
        let mut client = trezor_client::unique(false)?;
        client.init_device(Some(session_id))?;
        Ok(client)
    }

    /// Get the account which corresponds to our derivation path
    pub async fn get_address(&self) -> Result<Address, TrezorError> {
        self.get_address_with_path(&self.derivation).await
    }

    /// Gets the account which corresponds to the provided derivation path
    pub async fn get_address_with_path(
        &self,
        derivation: &DerivationType,
    ) -> Result<Address, TrezorError> {
        let mut client = self.get_client(self.session_id.clone())?;

        let address_str = client.ethereum_get_address(Self::convert_path(derivation))?;

        let mut address = [0; 20];
        address.copy_from_slice(&hex::decode(&address_str[2..])?);

        Ok(Address::from(address))
    }

    /// Signs an Ethereum transaction (requires confirmation on the Trezor)
    pub async fn sign_tx(&self, tx: &TypedTransaction) -> Result<Signature, TrezorError> {
        let mut client = self.get_client(self.session_id.clone())?;

        let arr_path = Self::convert_path(&self.derivation);

        let transaction = TrezorTransaction::load(tx)?;

        let chain_id = tx.chain_id().map(|id| id.as_u64()).unwrap_or(self.chain_id);

        let signature = match tx {
            TypedTransaction::Eip2930(_) | TypedTransaction::Legacy(_) => client.ethereum_sign_tx(
                arr_path,
                transaction.nonce,
                transaction.gas_price,
                transaction.gas,
                transaction.to,
                transaction.value,
                transaction.data,
                chain_id,
            )?,
            TypedTransaction::Eip1559(eip1559_tx) => client.ethereum_sign_eip1559_tx(
                arr_path,
                transaction.nonce,
                transaction.gas,
                transaction.to,
                transaction.value,
                transaction.data,
                chain_id,
                transaction.max_fee_per_gas,
                transaction.max_priority_fee_per_gas,
                transaction.access_list,
            )?,
            #[cfg(feature = "optimism")]
            TypedTransaction::OptimismDeposited(tx) => {
                trezor_client::client::Signature { r: 0.into(), s: 0.into(), v: 0 }
            }
        };

        Ok(Signature { r: signature.r, s: signature.s, v: signature.v })
    }

    /// Signs an ethereum personal message
    pub async fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Result<Signature, TrezorError> {
        let message = message.as_ref();
        let mut client = self.get_client(self.session_id.clone())?;
        let apath = Self::convert_path(&self.derivation);

        let signature = client.ethereum_sign_message(message.into(), apath)?;

        Ok(Signature { r: signature.r, s: signature.s, v: signature.v })
    }

    /// Signs an EIP712 encoded domain separator and message
    pub async fn sign_typed_struct<T>(&self, payload: &T) -> Result<Signature, TrezorError>
    where
        T: Eip712,
    {
        unimplemented!()
    }

    // helper which converts a derivation path to [u32]
    fn convert_path(derivation: &DerivationType) -> Vec<u32> {
        let derivation = derivation.to_string();
        let elements = derivation.split('/').skip(1).collect::<Vec<_>>();
        let depth = elements.len();

        let mut path = vec![];
        for derivation_index in elements {
            let hardened = derivation_index.contains('\'');
            let mut index = derivation_index.replace('\'', "").parse::<u32>().unwrap();
            if hardened {
                index |= 0x80000000;
            }
            path.push(index);
        }

        path
    }
}

#[cfg(all(test, feature = "trezor"))]
mod tests {
    use super::*;
    use crate::Signer;
    use ethers_core::types::{
        transaction::eip2930::{AccessList, AccessListItem},
        Address, Eip1559TransactionRequest, TransactionRequest, I256, U256,
    };
    use std::str::FromStr;

    #[tokio::test]
    #[ignore]
    // Replace this with your ETH addresses.
    async fn test_get_address() {
        // Instantiate it with the default trezor derivation path
        let trezor =
            TrezorEthereum::new(DerivationType::TrezorLive(1), 1, Some(PathBuf::from("randomdir")))
                .await
                .unwrap();
        assert_eq!(
            trezor.get_address().await.unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
        assert_eq!(
            trezor.get_address_with_path(&DerivationType::TrezorLive(0)).await.unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_tx() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();

        // approve uni v2 router 0xff
        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let tx_req = TransactionRequest::new()
            .to("2ed7afa17473e17ac59908f088b4371d28585476".parse::<Address>().unwrap())
            .gas(1000000)
            .gas_price(400e9 as u64)
            .nonce(5)
            .data(data)
            .value(ethers_core::utils::parse_ether(100).unwrap())
            .into();
        let tx = trezor.sign_transaction(&tx_req).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_big_data_tx() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();

        // invalid data
        let big_data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_string()+ &"ff".repeat(1032*2) + "aa").unwrap();
        let tx_req = TransactionRequest::new()
            .to("2ed7afa17473e17ac59908f088b4371d28585476".parse::<Address>().unwrap())
            .gas(1000000)
            .gas_price(400e9 as u64)
            .nonce(5)
            .data(big_data)
            .value(ethers_core::utils::parse_ether(100).unwrap())
            .into();
        let tx = trezor.sign_transaction(&tx_req).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_empty_txes() {
        // Contract creation (empty `to`), requires data.
        // To test without the data field, we need to specify a `to` address.
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();
        {
            let tx_req = Eip1559TransactionRequest::new()
                .to("2ed7afa17473e17ac59908f088b4371d28585476".parse::<Address>().unwrap())
                .into();
            let tx = trezor.sign_transaction(&tx_req).await.unwrap();
        }
        {
            let tx_req = TransactionRequest::new()
                .to("2ed7afa17473e17ac59908f088b4371d28585476".parse::<Address>().unwrap())
                .into();
            let tx = trezor.sign_transaction(&tx_req).await.unwrap();
        }

        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        // Contract creation (empty `to`, with data) should show on the trezor device as:
        //  ` "0 Wei ETH
        //  ` new contract?"
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();
        {
            let tx_req = Eip1559TransactionRequest::new().data(data.clone()).into();
            let tx = trezor.sign_transaction(&tx_req).await.unwrap();
        }
        {
            let tx_req = TransactionRequest::new().data(data.clone()).into();
            let tx = trezor.sign_transaction(&tx_req).await.unwrap();
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_eip1559_tx() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();

        // approve uni v2 router 0xff
        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let lst = AccessList(vec![
            AccessListItem {
                address: "0x8ba1f109551bd432803012645ac136ddd64dba72".parse().unwrap(),
                storage_keys: vec![
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    "0x0000000000000000000000000000000000000000000000000000000000000042"
                        .parse()
                        .unwrap(),
                ],
            },
            AccessListItem {
                address: "0x2ed7afa17473e17ac59908f088b4371d28585476".parse().unwrap(),
                storage_keys: vec![
                    "0x0000000000000000000000000000000000000000000000000000000000000000"
                        .parse()
                        .unwrap(),
                    "0x0000000000000000000000000000000000000000000000000000000000000042"
                        .parse()
                        .unwrap(),
                ],
            },
        ]);

        let tx_req = Eip1559TransactionRequest::new()
            .to("2ed7afa17473e17ac59908f088b4371d28585476".parse::<Address>().unwrap())
            .gas(1000000)
            .max_fee_per_gas(400e9 as u64)
            .max_priority_fee_per_gas(400e9 as u64)
            .nonce(5)
            .data(data)
            .access_list(lst)
            .value(ethers_core::utils::parse_ether(100).unwrap())
            .into();

        let tx = trezor.sign_transaction(&tx_req).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_message() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1, None).await.unwrap();
        let message = "hello world";
        let sig = trezor.sign_message(message).await.unwrap();
        let addr = trezor.get_address().await.unwrap();
        sig.verify(message, addr).unwrap();
    }
}
