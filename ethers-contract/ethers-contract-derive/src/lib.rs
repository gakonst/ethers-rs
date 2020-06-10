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
/// # Examples
///
/// ```no_run
/// // ABI Path
/// abigen!(MyContract, "MyContract.json");
///
/// // HTTP(S) source
/// abigen!(MyContract, "https://my.domain.local/path/to/contract.json")
///
/// // Etherscan.io
/// abigen!(MyContract, "etherscan:0x0001020304050607080910111213141516171819");
/// abigen!(MyContract, "https://etherscan.io/address/0x0001020304050607080910111213141516171819");
///
/// // npmjs
/// abigen!(MyContract, "npm:@org/package@1.0.0/path/to/contract.json")
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
/// ```no_run
/// abigen!(
///     MyContract,
///     "path/to/MyContract.json",
///     methods {
///         myMethod(uint256,bool) as my_renamed_method;
///     },
///     event_derives (serde::Deserialize, serde::Serialize),
/// );
/// ```
#[proc_macro]
pub fn abigen(input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(input as Spanned<ContractArgs>);

    let span = args.span();
    expand(args.into_inner())
        .unwrap_or_else(|e| Error::new(span, format!("{:?}", e)).to_compile_error())
        .into()
}
