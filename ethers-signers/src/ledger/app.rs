#![allow(unused)]
use coins_ledger::{
    common::{APDUAnswer, APDUCommand, APDUData},
    transports::{Ledger, LedgerAsync},
};
use futures_util::lock::Mutex;

use ethers_core::{
    types::{
        Address, NameOrAddress, Signature, Transaction, TransactionRequest, TxHash, H256, U256,
    },
    utils::keccak256,
};
use std::convert::TryFrom;
use thiserror::Error;

use super::types::*;

/// A Ledger Ethereum App.
///
/// This is a simple wrapper around the [Ledger transport](Ledger)
#[derive(Debug)]
pub struct LedgerEthereum {
    transport: Mutex<Ledger>,
    derivation: DerivationType,
    pub chain_id: Option<u64>,

    /// The ledger's address, instantiated at runtime
    pub address: Address,
}

impl LedgerEthereum {
    /// Instantiate the application by acquiring a lock on the ledger device.
    ///
    ///
    /// ```
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use ethers::signers::{Ledger, HDPath};
    ///
    /// let ledger = Ledger::new(HDPath::LedgerLive(0), Some(1)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(
        derivation: DerivationType,
        chain_id: Option<u64>,
    ) -> Result<Self, LedgerError> {
        let transport = Ledger::init().await?;
        let address = Self::get_address_with_path_transport(&transport, &derivation).await?;

        Ok(Self {
            transport: Mutex::new(transport),
            derivation,
            chain_id,
            address,
        })
    }

    /// Consume self and drop the ledger mutex
    pub fn close(self) {}

    /// Get the account which corresponds to our derivation path
    pub async fn get_address(&self) -> Result<Address, LedgerError> {
        self.get_address_with_path(&self.derivation).await
    }

    /// Gets the account which corresponds to the provided derivation path
    pub async fn get_address_with_path(
        &self,
        derivation: &DerivationType,
    ) -> Result<Address, LedgerError> {
        let data = APDUData::new(&Self::path_to_bytes(&derivation));
        let transport = self.transport.lock().await;
        Self::get_address_with_path_transport(&transport, derivation).await
    }

    async fn get_address_with_path_transport(
        transport: &Ledger,
        derivation: &DerivationType,
    ) -> Result<Address, LedgerError> {
        let data = APDUData::new(&Self::path_to_bytes(&derivation));

        let command = APDUCommand {
            ins: INS::GET_PUBLIC_KEY as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data,
            response_len: None,
        };

        let answer = transport.exchange(&command).await?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        let address = {
            // extract the address from the response
            let offset = 1 + result[0] as usize;
            let address = &result[offset + 1..offset + 1 + result[offset] as usize];
            std::str::from_utf8(address)?.parse::<Address>()?
        };

        Ok(address)
    }

    /// Returns the semver of the Ethereum ledger app
    pub async fn version(&self) -> Result<String, LedgerError> {
        let transport = self.transport.lock().await;

        let command = APDUCommand {
            ins: INS::GET_APP_CONFIGURATION as u8,
            p1: P1::NON_CONFIRM as u8,
            p2: P2::NO_CHAINCODE as u8,
            data: APDUData::new(&[]),
            response_len: None,
        };

        let answer = transport.exchange(&command).await?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        Ok(format!("{}.{}.{}", result[1], result[2], result[3]))
    }

    /// Signs an Ethereum transaction (requires confirmation on the ledger)
    pub async fn sign_tx(
        &self,
        tx: &TransactionRequest,
        chain_id: Option<u64>,
    ) -> Result<Signature, LedgerError> {
        let mut payload = Self::path_to_bytes(&self.derivation);
        payload.extend_from_slice(tx.rlp(chain_id).as_ref());
        self.sign_payload(INS::SIGN, payload).await
    }

    /// Signs an ethereum personal message
    pub async fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Result<Signature, LedgerError> {
        let message = message.as_ref();

        let mut payload = Self::path_to_bytes(&self.derivation);
        payload.extend_from_slice(&(message.len() as u32).to_be_bytes());
        payload.extend_from_slice(message);

        self.sign_payload(INS::SIGN_PERSONAL_MESSAGE, payload).await
    }

    // Helper function for signing either transaction data or personal messages
    async fn sign_payload(
        &self,
        command: INS,
        mut payload: Vec<u8>,
    ) -> Result<Signature, LedgerError> {
        let transport = self.transport.lock().await;
        let mut command = APDUCommand {
            ins: command as u8,
            p1: P1_FIRST,
            p2: P2::NO_CHAINCODE as u8,
            data: APDUData::new(&[]),
            response_len: None,
        };

        let mut result = Vec::new();

        // Iterate in 255 byte chunks
        while !payload.is_empty() {
            let chunk_size = std::cmp::min(payload.len(), 255);
            let data = payload.drain(0..chunk_size).collect::<Vec<_>>();
            command.data = APDUData::new(&data);

            let answer = transport.exchange(&command).await?;
            result = answer
                .data()
                .ok_or(LedgerError::UnexpectedNullResponse)?
                .to_vec();

            // We need more data
            command.p1 = P1::MORE as u8;
        }

        let v = result[0] as u64;
        let r = H256::from_slice(&result[1..33]);
        let s = H256::from_slice(&result[33..]);
        Ok(Signature { v, r, s })
    }

    // helper which converts a derivation path to bytes
    fn path_to_bytes(derivation: &DerivationType) -> Vec<u8> {
        let derivation = derivation.to_string();
        let elements = derivation.split('/').skip(1).collect::<Vec<_>>();
        let depth = elements.len();

        let mut bytes = vec![depth as u8];
        for derivation_index in elements {
            let hardened = derivation_index.contains('\'');
            let mut index = derivation_index.replace("'", "").parse::<u32>().unwrap();
            if hardened {
                index |= 0x80000000;
            }

            bytes.extend(&index.to_be_bytes());
        }

        bytes
    }
}

#[cfg(all(test, feature = "ledger-tests"))]
mod tests {
    use super::*;
    use crate::Signer;
    use ethers::prelude::*;
    use rustc_hex::FromHex;
    use std::str::FromStr;

    #[tokio::test]
    #[ignore]
    // Replace this with your ETH addresses.
    async fn test_get_address() {
        // Instantiate it with the default ledger derivation path
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), None)
            .await
            .unwrap();
        assert_eq!(
            ledger.get_address().await.unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
        assert_eq!(
            ledger
                .get_address_with_path(&DerivationType::Legacy(0))
                .await
                .unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_tx() {
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), None)
            .await
            .unwrap();

        // approve uni v2 router 0xff
        let data = "095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".from_hex::<Vec<u8>>().unwrap();

        let tx_req = TransactionRequest::new()
            .send_to_str("2ed7afa17473e17ac59908f088b4371d28585476")
            .unwrap()
            .gas(1000000)
            .gas_price(400e9 as u64)
            .nonce(5)
            .data(data)
            .value(ethers_core::utils::parse_ether(100).unwrap());
        let tx = ledger.sign_transaction(&tx_req).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_version() {
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), None)
            .await
            .unwrap();

        let version = ledger.version().await.unwrap();
        assert_eq!(version, "1.3.7");
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_message() {
        let ledger = LedgerEthereum::new(DerivationType::Legacy(0), None)
            .await
            .unwrap();
        let message = "hello world";
        let sig = ledger.sign_message(message).await.unwrap();
        let addr = ledger.get_address().await.unwrap();
        sig.verify(message, addr).unwrap();
    }
}
