//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code, unused)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

use abigen::Contracts;

pub(crate) mod abi_ty;
mod abigen;
mod event;
mod spanned;
pub(crate) mod utils;

/// Proc macro to generate type-safe bindings to a contract(s). This macro
/// accepts one or more Ethereum contract ABI or a path. Note that this path is
/// rooted in the crate's root `CARGO_MANIFEST_DIR`.
///
/// # Examples
///
/// ```ignore
/// # use ethers_contract_derive::abigen;
/// // ABI Path
/// abigen!(MyContract, "MyContract.json");
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
/// ```
///
/// Note that Etherscan rate-limits requests to their API, to avoid this an
/// `ETHERSCAN_API_KEY` environment variable can be set. If it is, it will use
/// that API key when retrieving the contract ABI.
///
/// Currently the proc macro accepts additional parameters to configure some
/// aspects of the code generation. Specifically it accepts:
/// - `methods`: A list of mappings from method signatures to method names
///   allowing methods names to be explicitely set for contract methods. This
///   also provides a workaround for generating code for contracts with multiple
///   methods with the same name.
/// - `event_derives`: A list of additional derives that should be added to
///   contract event structs and enums.
///
/// # Example
///
/// ```ignore
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
///
/// `abigen!` supports multiple abigen definitions separated by a semicolon `;`
/// This is useful if the contracts use ABIEncoderV2 structs. In which case
/// `abigen!` bundles all type duplicates so that all rust contracts also use
/// the same rust types.
///
/// # Example Multiple contracts
/// ```ignore
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize);
///
///     MyOtherContract,
///     "path/to/MyOtherContract.json",
///     event_derives (serde::Deserialize, serde::Serialize);
/// );
/// ```
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let contracts = parse_macro_input!(input as Contracts);

    contracts
        .expand()
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derives the `Tokenizable` trait for the labeled type.
///
/// This derive macro automatically adds a type bound `field: Tokenizable` for
/// each field type.
#[proc_macro_derive(EthAbiType)]
pub fn derive_abi_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    TokenStream::from(abi_ty::derive_tokenizeable_impl(&input))
}

/// Derives the `EthEvent` and `Tokenizeable` trait for the labeled type.
///
/// Additional arguments can be specified using the `#[ethevent(...)]`
/// attribute:
///
/// For the struct:
///
/// - `name`, `name = "..."`: Overrides the generated `EthEvent` name, default
///   is the
///  struct's name.
/// - `signature`, `signature = "..."`: The signature as hex string to override
///   the
///  event's signature.
/// - `abi`, `abi = "..."`: The ABI signature for the event this event's data
///   corresponds to.
///  The `abi` should be solidity event definition or a tuple of the event's
/// types in case the  event has non elementary (other `EthAbiType`) types as
/// members
/// - `anonymous`: A flag to mark this as an anonymous event
///
/// For fields:
///
/// - `indexed`: flag to mark a field as an indexed event input
/// - `name`: override the name of an indexed event input, default is the rust
///   field name
///
/// # Example
/// ```ignore
/// # use ethers_core::types::Address;
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
    TokenStream::from(event::derive_eth_event_impl(input))
}
