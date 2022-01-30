//! Transaction types
use super::{extract_chain_id, rlp_opt, NUM_TX_FIELDS};
use crate::{
    types::{Address, Bytes, NameOrAddress, Signature, H256, U256, U64},
    utils::keccak256,
};

use rlp::{Decodable, RlpStream};
use serde::{Deserialize, Serialize};

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

    /// Transfered value (None for no transfer)
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
    #[serde(skip_serializing_if = "Option::is_none")]
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
        rlp.begin_list(NUM_TX_FIELDS);
        self.rlp_base(&mut rlp);

        // Only hash the 3 extra fields when preparing the
        // data to sign if chain_id is present
        rlp_opt(&mut rlp, &self.chain_id);
        rlp.append(&0u8);
        rlp.append(&0u8);
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
    fn decode_unsigned_rlp_base(
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

        txn.to = Some(rlp.at(*offset)?.as_val()?);
        *offset += 1;
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

        // If a signed transaction is passed to this method, the chainid would be set to the v value
        // of the signature.
        if let Ok(chainid) = rlp.at(offset)?.as_val() {
            txn.chain_id = Some(chainid);
        }

        // parse the last two elements so we return an error if a signed transaction is passed
        let _first_zero: u8 = rlp.at(offset + 1)?.as_val()?;
        let _second_zero: u8 = rlp.at(offset + 2)?.as_val()?;
        Ok(txn)
    }

    /// Decodes the given RLP into a transaction, attempting to decode its signature as well.
    pub fn decode_signed_rlp(rlp: &rlp::Rlp) -> Result<(Self, Signature), rlp::DecoderError> {
        let mut offset = 0;
        let mut txn = Self::decode_unsigned_rlp_base(rlp, &mut offset)?;

        let v = rlp.at(offset)?.as_val()?;
        // populate chainid from v
        txn.chain_id = extract_chain_id(v);
        offset += 1;
        let r = rlp.at(offset)?.as_val()?;
        offset += 1;
        let s = rlp.at(offset)?.as_val()?;

        let sig = Signature { r, s, v };
        Ok((txn, sig))
    }
}

impl Decodable for TransactionRequest {
    /// Decodes the given RLP into a transaction request, ignoring the signature if populated
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        TransactionRequest::decode_unsigned_rlp(rlp)
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
#[cfg(not(feature = "celo"))]
mod tests {
    use crate::types::Signature;
    use rlp::{Decodable, Rlp};

    use super::{Address, TransactionRequest, U256, U64};

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
}
