//! N.B.:
//! - crate names must not be [global paths](https://doc.rust-lang.org/reference/paths.html#path-qualifiers)
//!   since we must be able to override them internally, like in Multicall.
//!
//! - [`ETHERS_CRATE_NAMES`] cannot hold [`syn::Path`] because it is not [`Sync`], so the names must
//!   be parsed at every call.

use cargo_metadata::MetadataCommand;
use once_cell::sync::Lazy;
use std::path::PathBuf;

/// Crate names to use in Path resolution.
///
/// `(core, contract, providers)`
type CrateNames = (&'static str, &'static str, &'static str);

const DEFAULT_CRATE_NAMES: CrateNames = ("ethers::core", "ethers::contract", "ethers::providers");
const SUB_CRATE_NAMES: CrateNames = ("ethers_core", "ethers_contract", "ethers_providers");

/// See [`determine_ethers_crates`].
///
/// This ensures that the `MetadataCommand` is ran only once.
static ETHERS_CRATE_NAMES: Lazy<CrateNames> = Lazy::new(determine_ethers_crates);

/// Returns the `core` crate's [`Path`][syn::Path].
pub fn ethers_core_crate() -> syn::Path {
    syn::parse_str(ETHERS_CRATE_NAMES.0).unwrap()
}

/// Returns the `contract` crate's [`Path`][syn::Path].
pub fn ethers_contract_crate() -> syn::Path {
    syn::parse_str(ETHERS_CRATE_NAMES.1).unwrap()
}

/// Returns the `providers` crate's [`Path`][syn::Path].
pub fn ethers_providers_crate() -> syn::Path {
    syn::parse_str(ETHERS_CRATE_NAMES.2).unwrap()
}

/// Determines which crate paths to use by looking at the [metadata][cargo_metadata] of the project.
///
/// Returns `ethers_*` if *all* necessary dependencies are present, otherwise `ethers::*`.
fn determine_ethers_crates() -> CrateNames {
    // always defined in Cargo projects
    let manifest_dir: PathBuf =
        std::env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not defined").into();

    let lock_file = manifest_dir.join("Cargo.lock");
    let lock_file_existed = lock_file.exists();

    let names = crate_names_from_metadata(manifest_dir).unwrap_or(DEFAULT_CRATE_NAMES);

    // remove the lock file created from running the command
    if !lock_file_existed && lock_file.exists() {
        let _ = std::fs::remove_file(lock_file);
    }

    names
}

/// Runs [`cargo metadata`][MetadataCommand] from `manifest_dir` and determines the crate names to
/// use.
///
/// Returns `None` on any error or if no dependencies are found.
#[inline]
fn crate_names_from_metadata(manifest_dir: PathBuf) -> Option<CrateNames> {
    let metadata = MetadataCommand::new().current_dir(manifest_dir).exec().ok()?;
    let pkg = metadata.root_package()?;

    // HACK(mattsse): this is required in order to compile and test ethers' internal crates
    const INTERNAL_CRATES: [&str; 5] = [
        "ethers-contract",
        "ethers-derive-eip712",
        "ethers-signers",
        "ethers-middleware",
        "ethers-solc",
    ];
    let pkg_name = pkg.name.as_str();
    if INTERNAL_CRATES.contains(&pkg_name) {
        return Some(SUB_CRATE_NAMES)
    }

    let mut has_ethers_core = false;
    let mut has_ethers_contract = false;
    let mut has_ethers_providers = false;

    for dep in pkg.dependencies.iter() {
        match dep.name.as_str() {
            "ethers-core" => {
                has_ethers_core = true;
            }
            "ethers-contract" => {
                has_ethers_contract = true;
            }
            "ethers-providers" => {
                has_ethers_providers = true;
            }
            "ethers" => return None,
            _ => {}
        }
    }

    if has_ethers_core && has_ethers_contract && has_ethers_providers {
        Some(SUB_CRATE_NAMES)
    } else {
        None
    }
}
