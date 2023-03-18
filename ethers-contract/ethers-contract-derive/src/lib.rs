//! Procedural macros for generating type-safe bindings to an Ethereum smart contract.

#![deny(missing_docs, unsafe_code, unused_crate_dependencies)]
#![deny(rustdoc::broken_intra_doc_links)]

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
/// use ethers_contract_derive::abigen;
///
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

/// Derives the `AbiType` and all `Tokenizable` traits for the labeled type.
///
/// This derive macro automatically adds a type bound `field: Tokenizable` for
/// each field type.
#[proc_macro_derive(EthAbiType)]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match abi_ty::derive_tokenizeable_impl(&input) {
        Ok(tokens) => tokens,
        Err(err) => err.to_compile_error(),
    }
    .into()
}

/// Derives the `AbiEncode`, `AbiDecode` and traits for the labeled type.
///
/// This is an addition to `EthAbiType` that lacks the `AbiEncode`, `AbiDecode` implementation.
///
/// The reason why this is a separate macro is the `AbiEncode` / `AbiDecode` are `ethers`
/// generalized codec traits used for types, calls, etc. However, encoding/decoding a call differs
/// from the basic encoding/decoding, (`[selector + encode(self)]`)
///
/// # Example
///
/// ```ignore
/// use ethers_contract::{EthAbiCodec, EthAbiType};
/// use ethers_core::types::*;
///
/// #[derive(Debug, Clone, EthAbiType, EthAbiCodec)]
/// struct MyStruct {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
/// let val = MyStruct {..};
/// let bytes = val.encode();
/// let val = MyStruct::decode(&bytes).unwrap();
/// ```
#[proc_macro_derive(EthAbiCodec)]
pub fn derive_abi_codec(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    codec::derive_codec_impl(&input).into()
}

/// Derives `fmt::Display` trait and generates a convenient format for all the
/// underlying primitive types/tokens.
///
/// The fields of the structure are formatted comma separated, like `self.0,
/// self.1, self.2,...`
///
/// # Example
///
/// ```ignore
/// use ethers_contract::{EthDisplay, EthAbiType};
/// use ethers_core::types::*;
///
/// #[derive(Debug, Clone, EthAbiType, EthDisplay)]
/// struct MyStruct {
///     addr: Address,
///     old_value: String,
///     new_value: String,
///     h: H256,
///     arr_u8: [u8; 32],
///     arr_u16: [u16; 32],
///     v: Vec<u8>,
/// }
/// let val = MyStruct {..};
/// format!("{}", val);
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

/// Derives the `EthEvent` and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[ethevent(...)]`
/// attribute:
///
/// For the struct:
///
/// - `name`, `name = "..."`: Overrides the generated `EthEvent` name, default is the
///  struct's name.
/// - `signature`, `signature = "..."`: The signature as hex string to override the
///  event's signature.
/// - `abi`, `abi = "..."`: The ABI signature for the event this event's data corresponds to.
///  The `abi` should be solidity event definition or a tuple of the event's
/// types in case the  event has non elementary (other `EthAbiType`) types as
/// members
/// - `anonymous`: A flag to mark this as an anonymous event
///
/// For fields:
///
/// - `indexed`: flag to mark a field as an indexed event input
/// - `name`: override the name of an indexed event input, default is the rust field name
///
/// # Example
/// ```ignore
/// use ethers_contract::EthCall;
/// use ethers_core::types::Address;
///
/// #[derive(Debug, EthAbiType)]
/// struct Inner {
///     inner: Address,
///     msg: String,
/// }
///
/// #[derive(Debug, EthEvent)]
/// #[ethevent(abi = "ValueChangedEvent((address,string),string)")]
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

/// Derives the `EthCall` and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[ethcall(...)]`
/// attribute:
///
/// For the struct:
///
/// - `name`, `name = "..."`: Overrides the generated `EthCall` function name, default is the
///  struct's name.
/// - `abi`, `abi = "..."`: The ABI signature for the function this call's data corresponds to.
///
///  NOTE: in order to successfully parse the `abi` (`<name>(<args>,...)`) the `<name`>
///   must match either the struct name or the name attribute: `#[ethcall(name ="<name>"]`
///
/// # Example
///
/// ```ignore
/// use ethers_contract::EthCall;
///
/// #[derive(Debug, Clone, EthCall)]
/// #[ethcall(name ="my_call")]
/// struct MyCall {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
/// assert_eq!(
///     MyCall::abi_signature().as_ref(),
///     "my_call(address,string,string)"
/// );
/// ```
///
/// # Example
///
/// Call with struct inputs
///
/// ```ignore
/// use ethers_core::abi::Address;
///
/// #[derive(Debug, Clone, PartialEq, EthAbiType)]
/// struct SomeType {
///     inner: Address,
///     msg: String,
/// }
///
/// #[derive(Debug, PartialEq, EthCall)]
/// #[ethcall(name = "foo", abi = "foo(address,(address,string),string)")]
/// struct FooCall {
///     old_author: Address,
///     inner: SomeType,
///     new_value: String,
/// }
///
/// assert_eq!(
///     FooCall::abi_signature().as_ref(),
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

/// Derives the `EthError` and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[etherror(...)]`
/// attribute:
///
/// For the struct:
///
/// - `name`, `name = "..."`: Overrides the generated `EthCall` function name, default is the
///  struct's name.
/// - `abi`, `abi = "..."`: The ABI signature for the function this call's data corresponds to.
///
///  NOTE: in order to successfully parse the `abi` (`<name>(<args>,...)`) the `<name`>
///   must match either the struct name or the name attribute: `#[ethcall(name ="<name>"]`
///
/// # Example
///
/// ```ignore
/// use ethers_contract::EthError;
///
/// #[derive(Debug, Clone, EthError)]
/// #[etherror(name ="my_error")]
/// struct MyError {
///     addr: Address,
///     old_value: String,
///     new_value: String,
/// }
/// assert_eq!(
///     MyCall::abi_signature().as_ref(),
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

/// EIP-712 derive macro.
///
/// This crate provides a derive macro `Eip712` that is used to encode a rust struct
/// into a payload hash, according to <https://eips.ethereum.org/EIPS/eip-712>
///
/// The trait used to derive the macro is found in `ethers_core::transaction::eip712::Eip712`
/// Both the derive macro and the trait must be in context when using
///
/// This derive macro requires the `#[eip712]` attributes to be included
/// for specifying the domain separator used in encoding the hash.
///
/// NOTE: In addition to deriving `Eip712` trait, the `EthAbiType` trait must also be derived.
/// This allows the struct to be parsed into `ethers_core::abi::Token` for encoding.
///
/// # Optional Eip712 Parameters
///
/// The only optional parameter is `salt`, which accepts a string
/// that is hashed using keccak256 and stored as bytes.
///
/// # Example Usage
///
/// ```ignore
/// use ethers_contract::EthAbiType;
/// use ethers_derive_eip712::*;
/// use ethers_core::types::{transaction::eip712::Eip712, H160};
///
/// #[derive(Debug, Eip712, EthAbiType)]
/// #[eip712(
///     name = "Radicle",
///     version = "1",
///     chain_id = 1,
///     verifying_contract = "0x0000000000000000000000000000000000000000"
///     // salt is an optional parameter
///     salt = "my-unique-spice"
/// )]
/// pub struct Puzzle {
///     pub organization: H160,
///     pub contributor: H160,
///     pub commit: String,
///     pub project: String,
/// }
///
/// let puzzle = Puzzle {
///     organization: "0000000000000000000000000000000000000000"
///         .parse::<H160>()
///         .expect("failed to parse address"),
///     contributor: "0000000000000000000000000000000000000000"
///         .parse::<H160>()
///         .expect("failed to parse address"),
///     commit: "5693b7019eb3e4487a81273c6f5e1832d77acb53".to_string(),
///     project: "radicle-reward".to_string(),
/// };
///
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
