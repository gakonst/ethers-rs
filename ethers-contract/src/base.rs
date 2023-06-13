use crate::ContractInstance;
pub use ethers_core::abi::AbiError;
use ethers_core::{
    abi::{Abi, Detokenize, Error, Event, Function, FunctionExt, RawLog, Token, Tokenize},
    types::{Address, Bytes, Selector, H256},
};
use ethers_providers::Middleware;
use std::{
    borrow::Borrow,
    collections::{BTreeMap, HashMap},
    fmt::Debug,
    hash::Hash,
};

/// A reduced form of `Contract` which just takes the `abi` and produces
/// ABI encoded data for its functions.
#[derive(Debug, Clone)]
pub struct BaseContract {
    pub(crate) abi: Abi,

    /// A mapping from method signature to a name-index pair for accessing
    /// functions in the contract ABI. This is used to avoid allocation when
    /// searching for matching functions by signature.
    // Adapted from: <https://github.com/gnosis/ethcontract-rs/blob/master/src/contract.rs>
    pub methods: HashMap<Selector, (String, usize)>,
}

impl From<Abi> for BaseContract {
    /// Creates a new `BaseContract` from the abi.
    fn from(abi: Abi) -> Self {
        let methods = create_mapping(&abi.functions, |function| function.selector());
        Self { abi, methods }
    }
}

impl BaseContract {
    /// Returns the ABI encoded data for the provided function and arguments
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `encode_with_selector`
    pub fn encode<T: Tokenize>(&self, name: &str, args: T) -> Result<Bytes, AbiError> {
        let function = self.abi.function(name)?;
        encode_function_data(function, args)
    }

    /// Returns the ABI encoded data for the provided function selector and arguments
    pub fn encode_with_selector<T: Tokenize>(
        &self,
        signature: Selector,
        args: T,
    ) -> Result<Bytes, AbiError> {
        let function = self.get_from_signature(signature)?;
        encode_function_data(function, args)
    }

    /// Decodes the provided ABI encoded function arguments with the selected function name.
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `decode_with_selector`
    pub fn decode<D: Detokenize, T: AsRef<[u8]>>(
        &self,
        name: &str,
        bytes: T,
    ) -> Result<D, AbiError> {
        let function = self.abi.function(name)?;
        decode_function_data(function, bytes, true)
    }

    /// Decodes the provided ABI encoded function arguments with the selected function name.
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `decode_with_selector`
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_raw<T: AsRef<[u8]>>(&self, name: &str, bytes: T) -> Result<Vec<Token>, AbiError> {
        let function = self.abi.function(name)?;
        decode_function_data_raw(function, bytes, true)
    }

    /// Decodes the provided ABI encoded function output with the selected function name.
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `decode_with_selector`
    pub fn decode_output<D: Detokenize, T: AsRef<[u8]>>(
        &self,
        name: &str,
        bytes: T,
    ) -> Result<D, AbiError> {
        let function = self.abi.function(name)?;
        decode_function_data(function, bytes, false)
    }

    /// Decodes the provided ABI encoded function output with the selected function name.
    ///
    /// If the function exists multiple times and you want to use one of the overloaded
    /// versions, consider using `decode_with_selector`
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_output_raw<T: AsRef<[u8]>>(
        &self,
        name: &str,
        bytes: T,
    ) -> Result<Vec<Token>, AbiError> {
        let function = self.abi.function(name)?;
        decode_function_data_raw(function, bytes, false)
    }

    /// Decodes for a given event name, given the `log.topics` and
    /// `log.data` fields from the transaction receipt
    pub fn decode_event<D: Detokenize>(
        &self,
        name: &str,
        topics: Vec<H256>,
        data: Bytes,
    ) -> Result<D, AbiError> {
        let event = self.abi.event(name)?;
        decode_event(event, topics, data)
    }

    /// Decodes for a given event name, given the `log.topics` and
    /// `log.data` fields from the transaction receipt
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_event_raw(
        &self,
        name: &str,
        topics: Vec<H256>,
        data: Bytes,
    ) -> Result<Vec<Token>, AbiError> {
        let event = self.abi.event(name)?;
        decode_event_raw(event, topics, data)
    }

    /// Decodes the provided ABI encoded bytes with the selected function selector
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_with_selector_raw<T: AsRef<[u8]>>(
        &self,
        signature: Selector,
        bytes: T,
    ) -> Result<Vec<Token>, AbiError> {
        let function = self.get_from_signature(signature)?;
        decode_function_data_raw(function, bytes, true)
    }

    /// Decodes the provided ABI encoded bytes with the selected function selector
    pub fn decode_with_selector<D: Detokenize, T: AsRef<[u8]>>(
        &self,
        signature: Selector,
        bytes: T,
    ) -> Result<D, AbiError> {
        let function = self.get_from_signature(signature)?;
        decode_function_data(function, bytes, true)
    }

    /// Decodes the provided ABI encoded input bytes
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_input_raw<T: AsRef<[u8]>>(&self, bytes: T) -> Result<Vec<Token>, AbiError> {
        let function = self.get_fn_from_input(bytes.as_ref())?;
        decode_function_data_raw(function, bytes, true)
    }

    /// Decodes the provided ABI encoded input bytes
    pub fn decode_input<D: Detokenize, T: AsRef<[u8]>>(&self, bytes: T) -> Result<D, AbiError> {
        let function = self.get_fn_from_input(bytes.as_ref())?;
        decode_function_data(function, bytes, true)
    }

    /// Decode the provided ABI encoded bytes as the output of the provided
    /// function selector
    pub fn decode_output_with_selector<D: Detokenize, T: AsRef<[u8]>>(
        &self,
        signature: Selector,
        bytes: T,
    ) -> Result<D, AbiError> {
        let function = self.get_from_signature(signature)?;
        decode_function_data(function, bytes, false)
    }

    /// Decodes the provided ABI encoded bytes with the selected function selector
    ///
    /// Returns a [`Token`] vector, which lets you decode function arguments dynamically
    /// without knowing the return type.
    pub fn decode_output_with_selector_raw<T: AsRef<[u8]>>(
        &self,
        signature: Selector,
        bytes: T,
    ) -> Result<Vec<Token>, AbiError> {
        let function = self.get_from_signature(signature)?;
        decode_function_data_raw(function, bytes, false)
    }

    fn get_fn_from_input(&self, input: &[u8]) -> Result<&Function, AbiError> {
        let sig: [u8; 4] = input
            .get(0..4)
            .ok_or(AbiError::WrongSelector)?
            .try_into()
            .map_err(|_e| AbiError::WrongSelector)?;
        self.get_from_signature(sig)
    }

    fn get_from_signature(&self, signature: Selector) -> Result<&Function, AbiError> {
        Ok(self
            .methods
            .get(&signature)
            .map(|(name, index)| &self.abi.functions[name][*index])
            .ok_or_else(|| Error::InvalidName(hex::encode(signature)))?)
    }

    /// Returns a reference to the contract's ABI
    pub fn abi(&self) -> &Abi {
        &self.abi
    }

    /// Upgrades a `BaseContract` into a full fledged contract with an address and middleware.
    pub fn into_contract<B, M>(self, address: Address, client: B) -> ContractInstance<B, M>
    where
        B: Borrow<M>,
        M: Middleware,
    {
        ContractInstance::new(address, self, client)
    }
}

impl AsRef<Abi> for BaseContract {
    fn as_ref(&self) -> &Abi {
        self.abi()
    }
}

pub fn decode_event_raw(
    event: &Event,
    topics: Vec<H256>,
    data: Bytes,
) -> Result<Vec<Token>, AbiError> {
    Ok(event
        .parse_log(RawLog { topics, data: data.to_vec() })?
        .params
        .into_iter()
        .map(|param| param.value)
        .collect::<Vec<_>>())
}

pub fn decode_event<D: Detokenize>(
    event: &Event,
    topics: Vec<H256>,
    data: Bytes,
) -> Result<D, AbiError> {
    let tokens = decode_event_raw(event, topics, data)?;
    Ok(D::from_tokens(tokens)?)
}

/// Helper for ABI encoding arguments for a specific function
pub fn encode_function_data<T: Tokenize>(function: &Function, args: T) -> Result<Bytes, AbiError> {
    let tokens = args.into_tokens();
    Ok(function.encode_input(&tokens).map(Into::into)?)
}

/// Helper for ABI decoding raw data based on a function's input or output.
pub fn decode_function_data_raw<T: AsRef<[u8]>>(
    function: &Function,
    bytes: T,
    is_input: bool,
) -> Result<Vec<Token>, AbiError> {
    let bytes = bytes.as_ref();
    Ok(if is_input {
        if bytes.len() < 4 || bytes[..4] != function.selector() {
            return Err(AbiError::WrongSelector)
        }
        function.decode_input(&bytes[4..])?
    } else {
        function.decode_output(bytes)?
    })
}

/// Helper for ABI decoding raw data based on a function's input or output.
pub fn decode_function_data<D: Detokenize, T: AsRef<[u8]>>(
    function: &Function,
    bytes: T,
    is_input: bool,
) -> Result<D, AbiError> {
    let tokens = decode_function_data_raw(function, bytes, is_input)?;
    Ok(D::from_tokens(tokens)?)
}

/// Utility function for creating a mapping between a unique signature and a
/// name-index pair for accessing contract ABI items.
fn create_mapping<T, S, F>(
    elements: &BTreeMap<String, Vec<T>>,
    signature: F,
) -> HashMap<S, (String, usize)>
where
    S: Hash + Eq,
    F: Fn(&T) -> S,
{
    let signature = &signature;
    elements
        .iter()
        .flat_map(|(name, sub_elements)| {
            sub_elements
                .iter()
                .enumerate()
                .map(move |(index, element)| (signature(element), (name.to_owned(), index)))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_core::{abi::parse_abi, types::U256};

    #[test]
    fn can_parse_function_inputs() {
        let abi = BaseContract::from(parse_abi(&[
            "function approve(address _spender, uint256 value) external view returns (bool, bool)"
        ]).unwrap());

        let spender = "7a250d5630b4cf539739df2c5dacb4c659f2488d".parse::<Address>().unwrap();
        let amount = U256::MAX;

        let encoded = abi.encode("approve", (spender, amount)).unwrap();

        assert_eq!(hex::encode(&encoded), "095ea7b30000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488dffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");

        let (spender2, amount2): (Address, U256) = abi.decode("approve", encoded).unwrap();
        assert_eq!(spender, spender2);
        assert_eq!(amount, amount2);
    }

    #[test]
    fn test_sig_from_input() {
        let abi = BaseContract::from(parse_abi(&[
            "function approve(address _spender, uint256 value) external view returns (bool, bool)"
        ]).unwrap());
        let spender = "7a250d5630b4cf539739df2c5dacb4c659f2488d".parse::<Address>().unwrap();
        let amount = U256::MAX;
        let encoded = abi.encode("approve", (spender, amount)).unwrap();

        let decoded: (Address, U256) = abi.decode_input(&encoded).unwrap();
        assert_eq!(spender, decoded.0);
        assert_eq!(amount, decoded.1);
    }

    #[test]
    fn can_parse_events() {
        let abi = BaseContract::from(
            parse_abi(&[
                "event Approval(address indexed owner, address indexed spender, uint256 value)",
            ])
            .unwrap(),
        );

        let topics = vec![
            "8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
            "000000000000000000000000e4e60fdf9bf188fa57b7a5022230363d5bd56d08",
            "0000000000000000000000007a250d5630b4cf539739df2c5dacb4c659f2488d",
        ]
        .into_iter()
        .map(|hash| hash.parse::<H256>().unwrap())
        .collect::<Vec<_>>();
        let data = Bytes::from(
            hex::decode("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
                .unwrap(),
        );

        let (owner, spender, value): (Address, Address, U256) =
            abi.decode_event("Approval", topics, data).unwrap();
        assert_eq!(value, U256::MAX);
        assert_eq!(owner, "e4e60fdf9bf188fa57b7a5022230363d5bd56d08".parse::<Address>().unwrap());
        assert_eq!(spender, "7a250d5630b4cf539739df2c5dacb4c659f2488d".parse::<Address>().unwrap());
    }
}
