//! Implementation of procedural macro for generating type-safe bindings to an
//! ethereum smart contract.
#![deny(missing_docs, unsafe_code)]

mod spanned;
use spanned::Spanned;

mod abigen;
use abigen::{expand, ContractArgs};

use proc_macro::TokenStream;
use syn::{parse::Error, parse_macro_input};

/// Proc macro to generate type-safe bindings to a contract. This macro accepts
/// an Ethereum contract ABI or a path. Note that this path is rooted in
/// the crate's root `CARGO_MANIFEST_DIR`.
///
/// ```ignore
/// ethcontract::contract!("build/contracts/MyContract.json");
/// ```
///
/// Alternatively, other sources may be used, for full details consult the
/// `ethcontract-generate::source` documentation. Some basic examples:
///
/// ```ignore
/// // HTTP(S) source
/// ethcontract::contract!("https://my.domain.local/path/to/contract.json")
/// // Etherscan.io
/// ethcontract::contract!("etherscan:0x0001020304050607080910111213141516171819");
/// ethcontract::contract!("https://etherscan.io/address/0x0001020304050607080910111213141516171819");
/// // npmjs
/// ethcontract::contract!("npm:@org/package@1.0.0/path/to/contract.json")
/// ```
///
/// Note that Etherscan rate-limits requests to their API, to avoid this an
/// `ETHERSCAN_API_KEY` environment variable can be set. If it is, it will use
/// that API key when retrieving the contract ABI.
///
/// Currently the proc macro accepts additional parameters to configure some
/// aspects of the code generation. Specifically it accepts:
/// - `crate`: The name of the `ethcontract` crate. This is useful if the crate
///   was renamed in the `Cargo.toml` for whatever reason.
/// - `contract`: Override the contract name that is used for the generated
///   type. This is required when using sources that do not provide the contract
///   name in the artifact JSON such as Etherscan.
/// - `mod`: The name of the contract module to place generated code in. Note
///   that the root contract type gets re-exported in the context where the
///   macro was invoked. This defaults to the contract name converted into snake
///   case.
/// - `methods`: A list of mappings from method signatures to method names
///   allowing methods names to be explicitely set for contract methods. This
///   also provides a workaround for generating code for contracts with multiple
///   methods with the same name.
/// - `event_derives`: A list of additional derives that should be added to
///   contract event structs and enums.
///
/// Additionally, the ABI source can be preceeded by a visibility modifier such
/// as `pub` or `pub(crate)`. This visibility modifier is applied to both the
/// generated module and contract re-export. If no visibility modifier is
/// provided, then none is used for the generated code as well, making the
/// module and contract private to the scope where the macro was invoked.
///
/// ```ignore
/// ethcontract::contract!(
///     pub(crate) "build/contracts/MyContract.json",
///     crate = ethcontract_rename,
///     mod = my_contract_instance,
///     contract = MyContractInstance,
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
///
/// See [`ethcontract`](ethcontract) module level documentation for additional
/// information.
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let span = args.span();
    expand(args.into_inner())
        .unwrap_or_else(|e| Error::new(span, format!("{:?}", e)).to_compile_error())
        .into()
}
