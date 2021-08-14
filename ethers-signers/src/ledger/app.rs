#![allow(unused)]
use coins_ledger::{
    common::{APDUAnswer, APDUCommand, APDUData},
    transports::{Ledger, LedgerAsync},
};
use futures_executor::block_on;
use futures_util::lock::Mutex;

use ethers_core::{
    types::{
        transaction::eip2718::TypedTransaction, Address, NameOrAddress, Signature, Transaction,
        TransactionRequest, TxHash, H256, U256,
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
    pub(crate) chain_id: u64,
    pub(crate) address: Address,
}

impl LedgerEthereum {
    /// Instantiate the application by acquiring a lock on the ledger device.
    ///
    ///
    /// ```
    /// # async fn foo() -> Result<(), Box<dyn std::error::Error>> {
    /// use ethers::signers::{Ledger, HDPath};
    ///
    /// let ledger = Ledger::new(HDPath::LedgerLive(0), 1).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new(derivation: DerivationType, chain_id: u64) -> Result<Self, LedgerError> {
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

        let answer = block_on(transport.exchange(&command))?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        let address = {
            // extract the address from the response
            let offset = 1 + result[0] as usize;
            let address_str = &result[offset + 1..offset + 1 + result[offset] as usize];
            let mut address = [0; 20];
            address.copy_from_slice(&hex::decode(address_str)?);
            Address::from(address)
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

        let answer = block_on(transport.exchange(&command))?;
        let result = answer.data().ok_or(LedgerError::UnexpectedNullResponse)?;

        Ok(format!("{}.{}.{}", result[1], result[2], result[3]))
    }

    /// Signs an Ethereum transaction (requires confirmation on the ledger)
    pub async fn sign_tx(&self, tx: &TypedTransaction) -> Result<Signature, LedgerError> {
        let mut payload = Self::path_to_bytes(&self.derivation);
        payload.extend_from_slice(tx.rlp(self.chain_id).as_ref());
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

            let answer = block_on(transport.exchange(&command))?;
            result = answer
                .data()
                .ok_or(LedgerError::UnexpectedNullResponse)?
                .to_vec();

            // We need more data
            command.p1 = P1::MORE as u8;
        }

        let v = result[0] as u64;
        let r = U256::from_big_endian(&result[1..33]);
        let s = U256::from_big_endian(&result[33..]);
        Ok(Signature { r, s, v })
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

#[cfg(all(test, feature = "ledger"))]
mod tests {
    use super::*;
    use crate::Signer;
    use ethers::prelude::*;
    use std::str::FromStr;

    #[tokio::test]
    #[ignore]
    // Replace this with your ETH addresses.
    async fn test_get_address() {
        // Instantiate it with the default ledger derivation path
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), 1)
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
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), 1)
            .await
            .unwrap();

        // approve uni v2 router 0xff
        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let tx_req = TransactionRequest::new()
            .to("2ed7afa17473e17ac59908f088b4371d28585476"
                .parse::<Address>()
                .unwrap())
            .gas(1000000)
            .gas_price(400e9 as u64)
            .nonce(5)
            .data(data)
            .value(ethers_core::utils::parse_ether(100).unwrap())
            .into();
        let tx = ledger.sign_transaction(&tx_req).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_version() {
        let ledger = LedgerEthereum::new(DerivationType::LedgerLive(0), 1)
            .await
            .unwrap();

        let version = ledger.version().await.unwrap();
        assert_eq!(version, "1.3.7");
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_message() {
        let ledger = LedgerEthereum::new(DerivationType::Legacy(0), 1)
            .await
            .unwrap();
        let message = "hello world";
        let sig = ledger.sign_message(message).await.unwrap();
        let addr = ledger.get_address().await.unwrap();
        sig.verify(message, addr).unwrap();
    }
}
