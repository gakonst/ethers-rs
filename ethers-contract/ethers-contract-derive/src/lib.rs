//! Procedural macros for generating type-safe bindings to an Ethereum smart contract.

#![deny(missing_docs, unsafe_code, unused_crate_dependencies)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use abigen::Contracts;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod abi_ty;
mod abigen;
mod call;
pub(crate) mod calllike;
mod codec;
mod display;
mod eip712;
mod error;
mod event;
mod spanned;
pub(crate) mod utils;

/// Generates type-safe bindings to an Ethereum smart contract from its ABI.
///
/// All the accepted ABI sources are listed in the examples below and in [Source].
///
/// Note:
/// - relative paths are rooted in the crate's root (`CARGO_MANIFEST_DIR`).
/// - Environment variable interpolation is supported via `$` prefix, like
///   `"$CARGO_MANIFEST_DIR/contracts/c.json"`
/// - Etherscan rate-limits requests to their API. To avoid this, set the `ETHERSCAN_API_KEY`
///   environment variable.
///
/// Additionally, this macro accepts additional parameters to configure some aspects of the code
/// generation:
/// - `methods`: A list of mappings from method signatures to method names allowing methods names to
///   be explicitely set for contract methods. This also provides a workaround for generating code
///   for contracts with multiple methods with the same name.
/// - `derives`: A list of additional derive macros that are added to all the generated structs and
///   enums, after the default ones which are ([when applicable][tuple_derive_ref]):
///   * [PartialEq]
///   * [Eq]
///   * [Debug]
///   * [Default]
///   * [Hash]
///
/// [Source]: ethers_contract_abigen::Source
/// [tuple_derive_ref]: https://doc.rust-lang.org/stable/std/primitive.tuple.html#trait-implementations-1
///
/// # Examples
///
/// All the possible ABI sources:
///
/// ```ignore
/// # use ethers_contract_derive::abigen;
/// // ABI Path
/// abigen!(MyContract, "./MyContractABI.json");
///
/// // HTTP(S) source
/// abigen!(MyContract, "https://my.domain.local/path/to/contract.json");
///
/// // Etherscan.io
/// abigen!(MyContract, "etherscan:0x0001020304050607080910111213141516171819");
/// abigen!(MyContract, "https://etherscan.io/address/0x0001020304050607080910111213141516171819");
///
/// // npmjs
/// abigen!(MyContract, "npm:@org/package@1.0.0/path/to/contract.json");
///
/// // Human readable ABI
/// abigen!(MyContract, r"[
///     function setValue(string)
///     function getValue() external view returns (string)
///     event ValueChanged(address indexed author, string oldValue, string newValue)
/// ]");
/// ```
///
/// Specify additional parameters:
///
/// ```ignore
/// # use ethers_contract_derive::abigen;
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     derives(serde::Deserialize, serde::Serialize),
/// );
/// ```
///
/// Aliases for overloaded functions with no aliases provided in the `method` section are derived
/// automatically.
///
/// `abigen!` supports multiple abigen definitions separated by a semicolon `;`
/// This is useful if the contracts use ABIEncoderV2 structs. In which case
/// `abigen!` bundles all type duplicates so that all rust contracts also use
/// the same rust types.
///
/// ```ignore
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     derives(serde::Deserialize, serde::Serialize);
///
///     MyOtherContract,
///     "path/to/MyOtherContract.json",
///     derives(serde::Deserialize, serde::Serialize);
/// );
/// ```
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let contracts = parse_macro_input!(input as Contracts);
    match contracts.expand() {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`AbiType`] and all `Tokenizable` traits for the labeled type.
///
/// This derive macro adds a type bound `field: Tokenizable` for each field type.
///
/// [`AbiType`]: ethers_core::abi::AbiType
///
/// # Examples
///
/// ```
/// use ethers_contract_derive::EthAbiType;
/// use ethers_core::types::*;
/// use ethers_core::abi::{AbiType, ParamType};
///
/// #[derive(Clone, EthAbiType)]
/// struct MyStruct {
///     a: U256,
///     b: Address,
///     c: Bytes,
///     d: String,
///     e: H256,
/// }
///
/// assert_eq!(
///     MyStruct::param_type(),
///     ParamType::Tuple(vec![
///         ParamType::Uint(256),
///         ParamType::Address,
///         ParamType::Bytes,
///         ParamType::String,
///         ParamType::FixedBytes(32),
///     ]),
/// );
/// ```
#[proc_macro_derive(EthAbiType)]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match abi_ty::derive_tokenizeable_impl(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`AbiEncode`] and [`AbiDecode`] traits for the labeled type.
///
/// This is separate from other derive macros because this derives a generic codec implementation
/// for structs, while [`EthEvent`] and others derive a specialized implementation.
///
/// [`AbiEncode`]: ethers_core::abi::AbiEncode
/// [`AbiDecode`]: ethers_core::abi::AbiDecode
///
/// Note that this macro requires the `EthAbiType` macro to be derived or for the type to implement
/// `AbiType` and `Tokenizable`. The type returned by the `AbiType` implementation must be a
/// `Token::Tuple`, otherwise this macro's implementation of `AbiDecode` will panic at runtime.
///
/// # Examples
///
/// ```
/// use ethers_contract_derive::{EthAbiCodec, EthAbiType};
/// use ethers_core::types::Address;
/// use ethers_core::abi::{AbiDecode, AbiEncode};
///
/// #[derive(Clone, Debug, Default, PartialEq, EthAbiType, EthAbiCodec)]
/// struct MyStruct {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
///
/// let value = MyStruct::default();
/// let encoded = value.clone().encode();
/// let decoded = MyStruct::decode(&encoded).unwrap();
/// assert_eq!(decoded, value);
/// ```
#[proc_macro_derive(EthAbiCodec)]
pub fn derive_abi_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codec::derive_codec_impl(&input).into()
}

/// Derives the [`Display`] trait on structs by formatting each field based on its Ethereum type.
///
/// The final output is a comma separated list of the struct's fields, formatted as follows:
/// `self.0, self.1, self.2,...`
///
/// [`Display`]: std::fmt::Display
///
/// # Examples
///
/// ```
/// use ethers_contract_derive::{EthAbiType, EthDisplay};
/// use ethers_core::types::*;
///
/// #[derive(Clone, Default, EthAbiType, EthDisplay)]
/// struct MyStruct {
///     addr: Address,
///     old_value: String,
///     new_value: String,
///     h: H256,
///     arr_u8: [u8; 32],
///     arr_u16: [u16; 32],
///     v: Vec<u8>,
/// }
///
/// let s = MyStruct::default();
/// assert!(!format!("{s}").is_empty());
/// ```
#[proc_macro_derive(EthDisplay, attributes(ethdisplay))]
pub fn derive_eth_display(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match display::derive_eth_display_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`EthEvent`] and `Tokenizable` traits for the labeled type.
///
/// Additional arguments can be specified using the `#[ethevent(...)]` attribute:
///
/// For the struct:
///
/// - `name = "..."`: Overrides the generated event name. Defaults to the struct's name;
/// - `signature = "..."`: The signature as hex string to override the event's signature;
/// - `abi = "..."`: The ABI signature of the event. The `abi` should be a Solidity event definition
///   or a tuple of the event's types in case the event has non elementary (other `EthAbiType`)
///   types as members;
/// - `anonymous`: A flag to mark this as an anonymous event.
///
/// For fields:
///
/// - `indexed`: flag to mark a field as an indexed event input;
/// - `name = "..."`: override the name of an indexed event input, default is the rust field name.
///
/// [`EthEvent`]: https://docs.rs/ethers/latest/ethers/contract/trait.EthEvent.html
///
/// # Examples
///
/// ```ignore
/// use ethers_contract_derive::{EthAbiType, EthEvent};
/// use ethers_core::types::Address;
///
/// #[derive(EthAbiType)]
/// struct Inner {
///     inner: Address,
///     msg: String,
/// }
///
/// #[derive(EthEvent)]
/// #[ethevent(abi = "ValueChangedEvent(address,string,(address,string))")]
/// struct ValueChangedEvent {
///     #[ethevent(indexed, name = "_target")]
///     target: Address,
///     msg: String,
///     inner: Inner,
/// }
/// ```
#[proc_macro_derive(EthEvent, attributes(ethevent))]
pub fn derive_abi_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match event::derive_eth_event_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`EthCall`] and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[ethcall(...)]` attribute:
///
/// For the struct:
///
/// - `name = "..."`: Overrides the generated function name. Defaults to the struct's name;
/// - `abi = "..."`: The ABI signature of the function.
///
/// NOTE: in order to successfully parse the `abi` (`<name>(<args>,...)`), `<name>` must match
/// either the struct's name or the name attribute: `#[ethcall(name = "<name>"]`
///
/// [`EthCall`]: https://docs.rs/ethers/latest/ethers/contract/trait.EthCall.html
///
/// # Examples
///
/// ```ignore
/// use ethers_contract_derive::EthCall;
/// use ethers_core::abi::{Address, FunctionExt};
///
/// #[derive(EthCall)]
/// #[ethcall(name = "my_call")]
/// struct MyCall {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
///
/// assert_eq!(
///     MyCall::abi_signature(),
///     "my_call(address,string,string)"
/// );
/// ```
///
/// Call with struct inputs
///
/// ```ignore
/// use ethers_core::abi::{Address, EventExt};
/// use ethers_contract_derive::EthCall;
///
/// #[derive(Clone, PartialEq, EthAbiType)]
/// struct SomeType {
///     inner: Address,
///     msg: String,
/// }
///
/// #[derive(PartialEq, EthCall)]
/// #[ethcall(name = "foo", abi = "foo(address,(address,string),string)")]
/// struct FooCall {
///     old_author: Address,
///     inner: SomeType,
///     new_value: String,
/// }
///
/// assert_eq!(
///     FooCall::abi_signature(),
///     "foo(address,(address,string),string)"
/// );
/// ```
#[proc_macro_derive(EthCall, attributes(ethcall))]
pub fn derive_abi_call(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match call::derive_eth_call_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`EthError`] and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[etherror(...)]` attribute:
///
/// For the struct:
///
/// - `name = "..."`: Overrides the generated error name. Defaults to the struct's name;
/// - `abi = "..."`: The ABI signature of the error.
///
/// NOTE: in order to successfully parse the `abi` (`<name>(<args>,...)`), `<name>` must match
/// either the struct's name or the name attribute: `#[ethcall(name = "<name>"]`
///
/// [`EthError`]: https://docs.rs/ethers/latest/ethers/contract/trait.EthError.html
///
/// # Examples
///
/// ```ignore
/// use ethers_core::abi::{Address, ErrorExt};
/// use ethers_contract_derive::EthError;
///
/// #[derive(Clone, EthError)]
/// #[etherror(name = "my_error")]
/// struct MyError {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
///
/// assert_eq!(
///     MyError::abi_signature(),
///     "my_error(address,string,string)"
/// );
/// ```
#[proc_macro_derive(EthError, attributes(etherror))]
pub fn derive_abi_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match error::derive_eth_error_impl(input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the [`Eip712`] trait for the labeled type.
///
/// Encodes a Rust struct into a payload hash, according to [eip-712](https://eips.ethereum.org/EIPS/eip-712).
///
/// The following traits are required to be implemented for the struct:
/// - [`Clone`]
/// - [`Tokenizable`]: can be derived with [`EthAbiType`]
///
/// [`Tokenizable`]: ethers_core::abi::Tokenizable
///
/// # Attribute parameters
///
/// Required:
///
/// - `name = "..."`: The name of the EIP712 domain separator.
/// - `version = "..."`: The version of the EIP712 domain separator.
/// - `chain_id = ...`: The chain id of the EIP712 domain separator.
/// - `verifying_contract = "..."`: The verifying contract's address of the EIP712 domain separator.
///
/// Optional:
///
/// - `salt = "..."` or `raw_salt = "..."`: The salt of the EIP712 domain separator;
///   - `salt` is interpreted as UTF-8 bytes and hashed, while `raw_salt` is interpreted as a hex
///     string.
///
/// # Examples
///
/// ```
/// use ethers_contract_derive::{EthAbiType, Eip712};
/// use ethers_core::types::{transaction::eip712::Eip712, H160};
///
/// #[derive(Clone, Default, EthAbiType, Eip712)]
/// #[eip712(
///     name = "Radicle",
///     version = "1",
///     chain_id = 1,
///     verifying_contract = "0x0000000000000000000000000000000000000000",
///     // salt/raw_salt are optional parameters
///     salt = "my-unique-spice",
/// )]
/// pub struct Puzzle {
///     pub organization: H160,
///     pub contributor: H160,
///     pub commit: String,
///     pub project: String,
/// }
///
/// let puzzle = Puzzle::default();
/// let hash = puzzle.encode_eip712().unwrap();
/// ```
///
/// # Limitations
///
/// At the moment, the derive macro does not recursively encode nested Eip712 structs.
///
/// There is an Inner helper attribute `#[eip712]` for fields that will eventually be used to
/// determine if there is a nested eip712 struct. However, this work is not yet complete.
#[proc_macro_derive(Eip712, attributes(eip712))]
pub fn derive_eip712(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match eip712::impl_derive_eip712(&input) {
        Ok(tokens) => tokens,
        Err(e) => e.to_compile_error(),
    }
    .into()
}
