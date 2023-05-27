//! Transaction types
use super::{decode_to, extract_chain_id, rlp_opt, NUM_TX_FIELDS};
use crate::{
    types::{
        Address, Bytes, NameOrAddress, Signature, SignatureError, Transaction, H256, U256, U64,
    },
    utils::keccak256,
};

use rlp::{Decodable, RlpStream};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An error involving a transaction request.
#[derive(Debug, Error)]
pub enum RequestError {
    /// When decoding a transaction request from RLP
    #[error(transparent)]
    DecodingError(#[from] rlp::DecoderError),
    /// When recovering the address from a signature
    #[error(transparent)]
    RecoveryError(#[from] SignatureError),
}

/// Parameters for sending a transaction
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct TransactionRequest {
    /// Sender address or ENS name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,

    /// Recipient address (None for contract creation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<NameOrAddress>,

    /// Supplied gas (None for sensible default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<U256>,

    /// Gas price (None for sensible default)
    #[serde(rename = "gasPrice")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<U256>,

    /// Transferred value (None for no transfer)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<U256>,

    /// The compiled code of a contract OR the first 4 bytes of the hash of the
    /// invoked method signature and encoded parameters. For details see Ethereum Contract ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Bytes>,

    /// Transaction nonce (None for next available nonce)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<U256>,

    /// Chain ID (None for mainnet)
    #[serde(skip_serializing)]
    #[serde(default, rename = "chainId")]
    pub chain_id: Option<U64>,

    /////////////////  Celo-specific transaction fields /////////////////
    /// The currency fees are paid in (None for native currency)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_currency: Option<Address>,

    /// Gateway fee recipient (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee_recipient: Option<Address>,

    /// Gateway fee amount (None for no gateway fee paid)
    #[cfg(feature = "celo")]
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_fee: Option<U256>,
}

impl TransactionRequest {
    /// Creates an empty transaction request with all fields left empty
    pub fn new() -> Self {
        Self::default()
    }

    /// Convenience function for sending a new payment transaction to the receiver.
    pub fn pay<T: Into<NameOrAddress>, V: Into<U256>>(to: T, value: V) -> Self {
        TransactionRequest { to: Some(to.into()), value: Some(value.into()), ..Default::default() }
    }

    // Builder pattern helpers

    /// Sets the `from` field in the transaction to the provided value
    #[must_use]
    pub fn from<T: Into<Address>>(mut self, from: T) -> Self {
        self.from = Some(from.into());
        self
    }

    /// Sets the `to` field in the transaction to the provided value
    #[must_use]
    pub fn to<T: Into<NameOrAddress>>(mut self, to: T) -> Self {
        self.to = Some(to.into());
        self
    }

    /// Sets the `gas` field in the transaction to the provided value
    #[must_use]
    pub fn gas<T: Into<U256>>(mut self, gas: T) -> Self {
        self.gas = Some(gas.into());
        self
    }

    /// Sets the `gas_price` field in the transaction to the provided value
    #[must_use]
    pub fn gas_price<T: Into<U256>>(mut self, gas_price: T) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Sets the `value` field in the transaction to the provided value
    #[must_use]
    pub fn value<T: Into<U256>>(mut self, value: T) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Sets the `data` field in the transaction to the provided value
    #[must_use]
    pub fn data<T: Into<Bytes>>(mut self, data: T) -> Self {
        self.data = Some(data.into());
        self
    }

    /// Sets the `nonce` field in the transaction to the provided value
    #[must_use]
    pub fn nonce<T: Into<U256>>(mut self, nonce: T) -> Self {
        self.nonce = Some(nonce.into());
        self
    }

    /// Sets the `chain_id` field in the transaction to the provided value
    #[must_use]
    pub fn chain_id<T: Into<U64>>(mut self, chain_id: T) -> Self {
        self.chain_id = Some(chain_id.into());
        self
    }

    /// Hashes the transaction's data with the provided chain id
    pub fn sighash(&self) -> H256 {
        match self.chain_id {
            Some(_) => keccak256(self.rlp().as_ref()).into(),
            None => keccak256(self.rlp_unsigned().as_ref()).into(),
        }
    }

    /// Gets the transaction's RLP encoding, prepared with the chain_id and extra fields for
    /// signing. Assumes the chainid exists.
    pub fn rlp(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        if let Some(chain_id) = self.chain_id {
            rlp.begin_list(NUM_TX_FIELDS);
            self.rlp_base(&mut rlp);
            rlp.append(&chain_id);
            rlp.append(&0u8);
            rlp.append(&0u8);
        } else {
            rlp.begin_list(NUM_TX_FIELDS - 3);
            self.rlp_base(&mut rlp);
        }
        rlp.out().freeze().into()
    }

    /// Gets the unsigned transaction's RLP encoding
    pub fn rlp_unsigned(&self) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS - 3);
        self.rlp_base(&mut rlp);
        rlp.out().freeze().into()
    }

    /// Produces the RLP encoding of the transaction with the provided signature
    pub fn rlp_signed(&self, signature: &Signature) -> Bytes {
        let mut rlp = RlpStream::new();
        rlp.begin_list(NUM_TX_FIELDS);

        self.rlp_base(&mut rlp);

        // append the signature
        rlp.append(&signature.v);
        rlp.append(&signature.r);
        rlp.append(&signature.s);
        rlp.out().freeze().into()
    }

    pub(crate) fn rlp_base(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.nonce);
        rlp_opt(rlp, &self.gas_price);
        rlp_opt(rlp, &self.gas);

        #[cfg(feature = "celo")]
        self.inject_celo_metadata(rlp);

        rlp_opt(rlp, &self.to.as_ref());
        rlp_opt(rlp, &self.value);
        rlp_opt(rlp, &self.data.as_ref().map(|d| d.as_ref()));
    }

    /// Decodes the unsigned rlp, returning the transaction request and incrementing the counter
    /// passed as we are traversing the rlp list.
    pub(crate) fn decode_unsigned_rlp_base(
        rlp: &rlp::Rlp,
        offset: &mut usize,
    ) -> Result<Self, rlp::DecoderError> {
        let mut txn = TransactionRequest::new();
        txn.nonce = Some(rlp.at(*offset)?.as_val()?);
        *offset += 1;
        txn.gas_price = Some(rlp.at(*offset)?.as_val()?);
        *offset += 1;
        txn.gas = Some(rlp.at(*offset)?.as_val()?);
        *offset += 1;

        #[cfg(feature = "celo")]
        {
            txn.fee_currency = Some(rlp.at(*offset)?.as_val()?);
            *offset += 1;
            txn.gateway_fee_recipient = Some(rlp.at(*offset)?.as_val()?);
            *offset += 1;
            txn.gateway_fee = Some(rlp.at(*offset)?.as_val()?);
            *offset += 1;
        }

        txn.to = decode_to(rlp, offset)?.map(NameOrAddress::Address);
        txn.value = Some(rlp.at(*offset)?.as_val()?);
        *offset += 1;

        // finally we need to extract the data which will be encoded as another rlp
        let txndata = rlp::Rlp::new(rlp.at(*offset)?.as_raw()).data()?;
        txn.data = match txndata.len() {
            0 => None,
            _ => Some(Bytes::from(txndata.to_vec())),
        };
        *offset += 1;
        Ok(txn)
    }

    /// Decodes RLP into a transaction.
    pub fn decode_unsigned_rlp(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let mut offset = 0;
        let mut txn = Self::decode_unsigned_rlp_base(rlp, &mut offset)?;

        // If the transaction includes more info, like the chainid, as we serialize in `rlp`, this
        // will decode that value.
        if let Ok(chainid) = rlp.val_at(offset) {
            // If a signed transaction is passed to this method, the chainid would be set to the v
            // value of the signature.
            txn.chain_id = Some(chainid);
        }

        Ok(txn)
    }

    /// Decodes the given RLP into a transaction, attempting to decode its signature as well.
    pub fn decode_signed_rlp(rlp: &rlp::Rlp) -> Result<(Self, Signature), RequestError> {
        let mut offset = 0;
        let mut txn = Self::decode_unsigned_rlp_base(rlp, &mut offset)?;

        let v = rlp.at(offset)?.as_val()?;
        // populate chainid from v in case the signature follows EIP155
        txn.chain_id = extract_chain_id(v);
        offset += 1;
        let r = rlp.at(offset)?.as_val()?;
        offset += 1;
        let s = rlp.at(offset)?.as_val()?;

        let sig = Signature { r, s, v };
        txn.from = Some(sig.recover(txn.sighash())?);

        Ok((txn, sig))
    }
}

impl Decodable for TransactionRequest {
    /// Decodes the given RLP into a transaction request, ignoring the signature if populated
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        Self::decode_unsigned_rlp(rlp)
    }
}

impl From<&Transaction> for TransactionRequest {
    fn from(tx: &Transaction) -> TransactionRequest {
        TransactionRequest {
            from: Some(tx.from),
            to: tx.to.map(NameOrAddress::Address),
            gas: Some(tx.gas),
            gas_price: tx.gas_price,
            value: Some(tx.value),
            data: Some(Bytes(tx.input.0.clone())),
            nonce: Some(tx.nonce),
            chain_id: tx.chain_id.map(|x| U64::from(x.as_u64())),

            #[cfg(feature = "celo")]
            fee_currency: tx.fee_currency,

            #[cfg(feature = "celo")]
            gateway_fee_recipient: tx.gateway_fee_recipient,

            #[cfg(feature = "celo")]
            gateway_fee: tx.gateway_fee,
        }
    }
}

// Separate impl block for the celo-specific fields
#[cfg(feature = "celo")]
impl TransactionRequest {
    // modifies the RLP stream with the Celo-specific information
    fn inject_celo_metadata(&self, rlp: &mut RlpStream) {
        rlp_opt(rlp, &self.fee_currency);
        rlp_opt(rlp, &self.gateway_fee_recipient);
        rlp_opt(rlp, &self.gateway_fee);
    }

    /// Sets the `fee_currency` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[must_use]
    pub fn fee_currency<T: Into<Address>>(mut self, fee_currency: T) -> Self {
        self.fee_currency = Some(fee_currency.into());
        self
    }

    /// Sets the `gateway_fee` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[must_use]
    pub fn gateway_fee<T: Into<U256>>(mut self, gateway_fee: T) -> Self {
        self.gateway_fee = Some(gateway_fee.into());
        self
    }

    /// Sets the `gateway_fee_recipient` field in the transaction to the provided value
    #[cfg_attr(docsrs, doc(cfg(feature = "celo")))]
    #[must_use]
    pub fn gateway_fee_recipient<T: Into<Address>>(mut self, gateway_fee_recipient: T) -> Self {
        self.gateway_fee_recipient = Some(gateway_fee_recipient.into());
        self
    }
}

#[cfg(test)]
#[cfg(not(any(feature = "celo", feature = "optimism")))]
mod tests {
    use super::*;
    use crate::types::{transaction::eip2718::TypedTransaction, Bytes, NameOrAddress, Signature};
    use rlp::{Decodable, Rlp};
    use std::str::FromStr;

    #[test]
    fn encode_decode_rlp() {
        let tx = TransactionRequest::new()
            .nonce(3)
            .gas_price(1)
            .gas(25000)
            .to("b94f5374fce5edbc8e2a8697c15331677e6ebf0b".parse::<Address>().unwrap())
            .value(10)
            .data(vec![0x55, 0x44])
            .chain_id(1);

        // turn the rlp bytes encoding into a rlp stream and check that the decoding returns the
        // same struct
        let rlp_bytes = &tx.rlp().to_vec()[..];
        let got_rlp = Rlp::new(rlp_bytes);
        let txn_request = TransactionRequest::decode(&got_rlp).unwrap();

        // We compare the sighash rather than the specific struct
        assert_eq!(tx.sighash(), txn_request.sighash());
    }

    #[test]
    // test data from https://github.com/ethereum/go-ethereum/blob/b1e72f7ea998ad662166bcf23705ca59cf81e925/core/types/transaction_test.go#L40
    fn empty_sighash_check() {
        let tx = TransactionRequest::new()
            .nonce(0)
            .to("095e7baea6a6c7c4c2dfeb977efac326af552d87".parse::<Address>().unwrap())
            .value(0)
            .gas(0)
            .gas_price(0);

        let expected_sighash = "c775b99e7ad12f50d819fcd602390467e28141316969f4b57f0626f74fe3b386";
        let got_sighash = hex::encode(tx.sighash().as_bytes());
        assert_eq!(expected_sighash, got_sighash);
    }
    #[test]
    fn decode_unsigned_transaction() {
        let _res: TransactionRequest = serde_json::from_str(
            r#"{
    "gas":"0xc350",
    "gasPrice":"0x4a817c800",
    "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
    "input":"0x68656c6c6f21",
    "nonce":"0x15",
    "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
    "transactionIndex":"0x41",
    "value":"0xf3dbb76162000",
    "chain_id": "0x1"
  }"#,
        )
        .unwrap();
    }

    #[test]
    fn decode_known_rlp_goerli() {
        let tx = TransactionRequest::new()
            .nonce(70272)
            .from("fab2b4b677a4e104759d378ea25504862150256e".parse::<Address>().unwrap())
            .to("d1f23226fb4d2b7d2f3bcdd99381b038de705a64".parse::<Address>().unwrap())
            .value(0)
            .gas_price(1940000007)
            .gas(21000);

        let expected_rlp = hex::decode("f866830112808473a20d0782520894d1f23226fb4d2b7d2f3bcdd99381b038de705a6480801ca04bc89d41c954168afb4cbd01fe2e0f9fe12e3aa4665eefcee8c4a208df044b5da05d410fd85a2e31870ea6d6af53fafc8e3c1ae1859717c863cac5cff40fee8da4").unwrap();
        let (got_tx, _signature) =
            TransactionRequest::decode_signed_rlp(&Rlp::new(&expected_rlp)).unwrap();

        // intialization of TransactionRequests using new() uses the Default trait, so we just
        // compare the sighash and signed encoding instead.
        assert_eq!(got_tx.sighash(), tx.sighash());
    }

    #[test]
    fn decode_unsigned_rlp_no_chainid() {
        // unlike the corresponding transaction
        // 0x02c563d96acaf8c157d08db2228c84836faaf3dd513fc959a54ed4ca6c72573e, this doesn't have a
        // `from` field because the `from` field is only obtained via signature recovery
        let expected_tx = TransactionRequest::new()
            .to(Address::from_str("0xc7696b27830dd8aa4823a1cba8440c27c36adec4").unwrap())
            .gas(3_000_000)
            .gas_price(20_000_000_000u64)
            .value(0)
            .nonce(6306u64)
            .data(
                Bytes::from_str(
                    "0x91b7f5ed0000000000000000000000000000000000000000000000000000000000000372",
                )
                .unwrap(),
            );

        // manually stripped the signature off the end and modified length
        let expected_rlp = hex::decode("f8488218a28504a817c800832dc6c094c7696b27830dd8aa4823a1cba8440c27c36adec480a491b7f5ed0000000000000000000000000000000000000000000000000000000000000372").unwrap();
        let real_tx = TransactionRequest::decode(&Rlp::new(&expected_rlp)).unwrap();

        assert_eq!(real_tx, expected_tx);
    }

    #[test]
    fn test_eip155_encode() {
        let tx = TransactionRequest::new()
            .nonce(9)
            .to("3535353535353535353535353535353535353535".parse::<Address>().unwrap())
            .value(1000000000000000000u64)
            .gas_price(20000000000u64)
            .gas(21000)
            .chain_id(1);

        let expected_rlp = hex::decode("ec098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000080018080").unwrap();
        assert_eq!(expected_rlp, tx.rlp().to_vec());

        let expected_sighash =
            hex::decode("daf5a779ae972f972197303d7b574746c7ef83eadac0f2791ad23db92e4c8e53")
                .unwrap();

        assert_eq!(expected_sighash, tx.sighash().as_bytes().to_vec());
    }

    #[test]
    fn test_eip155_decode() {
        let tx = TransactionRequest::new()
            .nonce(9)
            .to("3535353535353535353535353535353535353535".parse::<Address>().unwrap())
            .value(1000000000000000000u64)
            .gas_price(20000000000u64)
            .gas(21000)
            .chain_id(1);

        let expected_hex = hex::decode("ec098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a764000080018080").unwrap();
        let expected_rlp = rlp::Rlp::new(expected_hex.as_slice());
        let decoded_transaction = TransactionRequest::decode(&expected_rlp).unwrap();
        assert_eq!(tx, decoded_transaction);
    }

    #[test]
    fn test_eip155_decode_signed() {
        let expected_signed_bytes = hex::decode("f86c098504a817c800825208943535353535353535353535353535353535353535880de0b6b3a76400008025a028ef61340bd939bc2195fe537567866003e1a15d3c71ff63e1590620aa636276a067cbe9d8997f761aecb703304b3800ccf555c9f3dc64214b297fb1966a3b6d83").unwrap();
        let expected_signed_rlp = rlp::Rlp::new(expected_signed_bytes.as_slice());
        let (decoded_tx, decoded_sig) =
            TransactionRequest::decode_signed_rlp(&expected_signed_rlp).unwrap();

        let expected_sig = Signature {
            v: 37,
            r: U256::from_dec_str(
                "18515461264373351373200002665853028612451056578545711640558177340181847433846",
            )
            .unwrap(),
            s: U256::from_dec_str(
                "46948507304638947509940763649030358759909902576025900602547168820602576006531",
            )
            .unwrap(),
        };
        assert_eq!(expected_sig, decoded_sig);
        assert_eq!(decoded_tx.chain_id, Some(U64::from(1)));
    }

    #[test]
    fn test_eip155_signing_decode_vitalik() {
        // Test vectors come from http://vitalik.ca/files/eip155_testvec.txt and
        // https://github.com/ethereum/go-ethereum/blob/master/core/types/transaction_signing_test.go
        // Tests that the rlp decoding properly extracts the from address
        let rlp_transactions =
            vec!["f864808504a817c800825208943535353535353535353535353535353535353535808025a0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116da0044852b2a670ade5407e78fb2863c51de9fcb96542a07186fe3aeda6bb8a116d",
                 "f864018504a817c80182a410943535353535353535353535353535353535353535018025a0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bcaa0489efdaa54c0f20c7adf612882df0950f5a951637e0307cdcb4c672f298b8bc6",
                 "f864028504a817c80282f618943535353535353535353535353535353535353535088025a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5a02d7c5bef027816a800da1736444fb58a807ef4c9603b7848673f7e3a68eb14a5",
                 "f865038504a817c803830148209435353535353535353535353535353535353535351b8025a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4e0a02a80e1ef1d7842f27f2e6be0972bb708b9a135c38860dbe73c27c3486c34f4de",
                 "f865048504a817c80483019a28943535353535353535353535353535353535353535408025a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c063a013600b294191fc92924bb3ce4b969c1e7e2bab8f4c93c3fc6d0a51733df3c060",
                 "f865058504a817c8058301ec309435353535353535353535353535353535353535357d8025a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1a04eebf77a833b30520287ddd9478ff51abbdffa30aa90a8d655dba0e8a79ce0c1",
                 "f866068504a817c80683023e3894353535353535353535353535353535353535353581d88025a06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2fa06455bf8ea6e7463a1046a0b52804526e119b4bf5136279614e0b1e8e296a4e2d",
                 "f867078504a817c807830290409435353535353535353535353535353535353535358201578025a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021a052f1a9b320cab38e5da8a8f97989383aab0a49165fc91c737310e4f7e9821021",
                 "f867088504a817c8088302e2489435353535353535353535353535353535353535358202008025a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c12a064b1702d9298fee62dfeccc57d322a463ad55ca201256d01f62b45b2e1c21c10",
                 "f867098504a817c809830334509435353535353535353535353535353535353535358202d98025a052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afba052f8f61201b2b11a78d6e866abc9c3db2ae8631fa656bfe5cb53668255367afb"];
        let rlp_transactions_bytes = rlp_transactions
            .iter()
            .map(|rlp_str| hex::decode(rlp_str).unwrap())
            .collect::<Vec<Vec<u8>>>();

        let raw_addresses = vec![
            "0xf0f6f18bca1b28cd68e4357452947e021241e9ce",
            "0x23ef145a395ea3fa3deb533b8a9e1b4c6c25d112",
            "0x2e485e0c23b4c3c542628a5f672eeab0ad4888be",
            "0x82a88539669a3fd524d669e858935de5e5410cf0",
            "0xf9358f2538fd5ccfeb848b64a96b743fcc930554",
            "0xa8f7aba377317440bc5b26198a363ad22af1f3a4",
            "0xf1f571dc362a0e5b2696b8e775f8491d3e50de35",
            "0xd37922162ab7cea97c97a87551ed02c9a38b7332",
            "0x9bddad43f934d313c2b79ca28a432dd2b7281029",
            "0x3c24d7329e92f84f08556ceb6df1cdb0104ca49f",
        ];

        let addresses = raw_addresses.iter().map(|addr| addr.parse::<Address>().unwrap().into());

        // decoding will do sender recovery and we don't expect any of these to error, so we should
        // check that the address matches for each decoded transaction
        let decoded_transactions = rlp_transactions_bytes.iter().map(|raw_tx| {
            TransactionRequest::decode_signed_rlp(&Rlp::new(raw_tx.as_slice())).unwrap().0
        });

        for (tx, from_addr) in decoded_transactions.zip(addresses) {
            let from_tx: NameOrAddress = tx.from.unwrap().into();
            assert_eq!(from_tx, from_addr);
        }
    }

    #[test]
    fn test_recover_legacy_tx() {
        let raw_tx = "f9015482078b8505d21dba0083022ef1947a250d5630b4cf539739df2c5dacb4c659f2488d880c46549a521b13d8b8e47ff36ab50000000000000000000000000000000000000000000066ab5a608bd00a23f2fe000000000000000000000000000000000000000000000000000000000000008000000000000000000000000048c04ed5691981c42154c6167398f95e8f38a7ff00000000000000000000000000000000000000000000000000000000632ceac70000000000000000000000000000000000000000000000000000000000000002000000000000000000000000c02aaa39b223fe8d0a0e5c4f27ead9083c756cc20000000000000000000000006c6ee5e31d828de241282b9606c8e98ea48526e225a0c9077369501641a92ef7399ff81c21639ed4fd8fc69cb793cfa1dbfab342e10aa0615facb2f1bcf3274a354cfe384a38d0cc008a11c2dd23a69111bc6930ba27a8";

        let data = hex::decode(raw_tx).unwrap();
        let rlp = Rlp::new(&data);
        let (tx, sig) = TypedTransaction::decode_signed(&rlp).unwrap();
        let recovered = sig.recover(tx.sighash()).unwrap();

        let expected: Address = "0xa12e1462d0ced572f396f58b6e2d03894cd7c8a4".parse().unwrap();
        assert_eq!(expected, recovered);
    }
}
