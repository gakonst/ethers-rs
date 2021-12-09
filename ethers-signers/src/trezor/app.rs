#![allow(unused)]
use trezor_client::client::{Trezor, AccessListItem as Trezor_AccessListItem};

use futures_executor::block_on;
use futures_util::lock::Mutex;

use ethers_core::{
    types::{
        transaction::{eip2718::TypedTransaction, eip712::Eip712},
        Address, NameOrAddress, Signature, Transaction, TransactionRequest, TxHash, H256, U256,
    },
    utils::keccak256,
};
use std::convert::TryFrom;
use thiserror::Error;

use super::types::*;

/// A Trezor Ethereum App.
///
/// This is a simple wrapper around the [Trezor transport](Trezor)
#[derive(Debug)]
pub struct TrezorEthereum {
    derivation: DerivationType,
    pub(crate) chain_id: u64,
    pub(crate) address: Address,
}

impl TrezorEthereum {
    fn get_client() -> Result<Trezor, TrezorError> {
        let mut client = trezor_client::unique(false)?;
        client.init_device()?;

        Ok(client)
    }

    pub async fn new(derivation: DerivationType, chain_id: u64) -> Result<Self, TrezorError> {
        // Check if reachable
        let address = Self::get_address_with_path(&derivation).await?;

        Ok(Self { derivation, chain_id, address })
    }

    /// Consume self and drop the Trezor mutex
    pub fn close(self) {}

    /// Get the account which corresponds to our derivation path
    pub async fn get_address(&self) -> Result<Address, TrezorError> {
        Ok(TrezorEthereum::get_address_with_path(&self.derivation).await?)
    }

    /// Gets the account which corresponds to the provided derivation path
    pub async fn get_address_with_path(
        derivation: &DerivationType,
    ) -> Result<Address, TrezorError> {
        let mut client = TrezorEthereum::get_client()?;

        let address_str = client.ethereum_get_address(Self::convert_path(derivation))?;

        let mut address = [0; 20];
        address.copy_from_slice(&hex::decode(&address_str[2..])?);


        Ok(Address::from(address))
    }

    /// Signs an Ethereum transaction (requires confirmation on the Trezor)
    pub async fn sign_tx(&self, tx: &TypedTransaction) -> Result<Signature, TrezorError> {

        let mut client = TrezorEthereum::get_client()?;

        let arr_path = Self::convert_path(&self.derivation);

        let mut nonce = [0 as u8; 32];
        let mut gas = [0 as u8; 32];
        let mut gas_price = [0 as u8; 32];
        let mut value = [0 as u8; 32];
        let mut max_fee_per_gas: Option<Vec<u8>> = None;
        let mut max_priority_fee_per_gas: Option<Vec<u8>> = None;

        tx.nonce().unwrap().to_big_endian(&mut nonce);
        tx.gas().unwrap().to_big_endian(&mut gas);
        tx.gas_price().unwrap().to_big_endian(&mut gas_price);
        tx.value().unwrap().to_big_endian(&mut value);

        let to: String = match tx.to().unwrap() {
            NameOrAddress::Name(_) => unimplemented!(),
            NameOrAddress::Address(value) => format!("0x{}", hex::encode(value)),
        };

        let signature = match tx {
            TypedTransaction::Eip2930(_) | TypedTransaction::Legacy(_) => {
                client.ethereum_sign_tx(
                    arr_path,
                    nonce[tx.nonce().unwrap().leading_zeros() as usize/8..].to_vec(),
                    gas_price[tx.gas_price().unwrap().leading_zeros() as usize/8..].to_vec(),
                    gas[tx.gas().unwrap().leading_zeros() as usize/8..].to_vec(),
                    to,
                    value[tx.value().unwrap().leading_zeros() as usize/8..].to_vec(),
                    tx.data().unwrap().to_vec(),
                    self.chain_id
                )?
            }
            TypedTransaction::Eip1559(eip1559_tx) => {
                let mut m_fpg = [0 as u8; 32];
                let mut m_pfpg = [0 as u8; 32];

                eip1559_tx.max_fee_per_gas.unwrap().to_big_endian(&mut m_fpg);
                eip1559_tx.max_priority_fee_per_gas.unwrap().to_big_endian(&mut m_pfpg);

                let mut trezor_access_list: Vec<Trezor_AccessListItem> = Vec::new();
                for item in &eip1559_tx.access_list.0 {

                    let address: String = format!("0x{}", hex::encode(item.address));
                    let mut storage_keys: Vec<Vec<u8>> = Vec::new();

                    for key in &item.storage_keys {
                        storage_keys.push(
                            key.as_bytes().to_vec()
                        )
                    }

                    trezor_access_list.push(
                        Trezor_AccessListItem {
                            address,
                            storage_keys
                        }
                    )
                }
                
                client.ethereum_sign_eip1559_tx(
                    arr_path,
                    nonce[tx.nonce().unwrap().leading_zeros() as usize/8..].to_vec(),
                    gas[tx.gas().unwrap().leading_zeros() as usize/8..].to_vec(),
                    to,
                    value[tx.value().unwrap().leading_zeros() as usize/8..].to_vec(),
                    tx.data().unwrap().to_vec(),
                    self.chain_id,
                    m_fpg[eip1559_tx.max_fee_per_gas.unwrap().leading_zeros() as usize / 8..].to_vec(),
                    m_pfpg[eip1559_tx.max_priority_fee_per_gas.unwrap().leading_zeros() as usize / 8..].to_vec(),
                    trezor_access_list
                )?
            }
        };

        Ok(Signature { r: signature.r, s: signature.s, v: signature.v })
    }

    /// Signs an ethereum personal message
    pub async fn sign_message<S: AsRef<[u8]>>(&self, message: S) -> Result<Signature, TrezorError> {
        let message = message.as_ref();
        let mut client = TrezorEthereum::get_client()?;
        let apath = Self::convert_path(&self.derivation);

        let signs = client.ethereum_sign_message(
            message.into(),
            apath
        )?;
        Ok(Signature { r: signs.r, s: signs.s, v: signs.v })
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
            let mut index = derivation_index.replace("'", "").parse::<u32>().unwrap();
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
    use ethers_contract::EthAbiType;
    use ethers_core::types::{
        transaction::eip2930::{AccessList, AccessListItem},
        transaction::eip712::Eip712, Address, TransactionRequest, Eip1559TransactionRequest, I256, U256,
    };
    use ethers_derive_eip712::*;
    use std::str::FromStr;

    #[derive(Debug, Clone, Eip712, EthAbiType)]
    #[eip712(
        name = "Eip712Test",
        version = "1",
        chain_id = 1,
        verifying_contract = "0x0000000000000000000000000000000000000001",
        salt = "eip712-test-75F0CCte"
    )]
    struct FooBar {
        foo: I256,
        bar: U256,
        fizz: Vec<u8>,
        buzz: [u8; 32],
        far: String,
        out: Address,
    }

    #[tokio::test]
    #[ignore]
    // Replace this with your ETH addresses.
    async fn test_get_address() {
        // Instantiate it with the default trezor derivation path
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(1), 1).await.unwrap();
        assert_eq!(
            trezor.get_address().await.unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
        assert_eq!(
            TrezorEthereum::get_address_with_path(&DerivationType::TrezorLive(0)).await.unwrap(),
            "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee".parse().unwrap()
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_tx() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1).await.unwrap();

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
    async fn test_sign_eip1559_tx() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1).await.unwrap();

        // approve uni v2 router 0xff
        let data = hex::decode("095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff").unwrap();

        let lst = AccessList(
            vec![AccessListItem {
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
            }
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
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1).await.unwrap();
        let message = "hello world";
        let sig = trezor.sign_message(message).await.unwrap();
        let addr = trezor.get_address().await.unwrap();
        sig.verify(message, addr).unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_sign_eip712_struct() {
        let trezor = TrezorEthereum::new(DerivationType::TrezorLive(0), 1u64).await.unwrap();

        let foo_bar = FooBar {
            foo: I256::from(10),
            bar: U256::from(20),
            fizz: b"fizz".to_vec(),
            buzz: keccak256("buzz"),
            far: String::from("space"),
            out: Address::from([0; 20]),
        };

        let sig = trezor.sign_typed_struct(&foo_bar).await.expect("failed to sign typed data");
        let foo_bar_hash = foo_bar.encode_eip712().unwrap();
        sig.verify(foo_bar_hash, trezor.address).unwrap();
    }
}
