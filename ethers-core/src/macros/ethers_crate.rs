//! N.B.:
//! - crate names must not be [global paths](https://doc.rust-lang.org/reference/paths.html#path-qualifiers)
//!   since we must be able to override them internally, like in Multicall.
//!
//! - [`ETHERS_CRATE_NAMES`] cannot hold [`syn::Path`] because it is not [`Sync`], so the names must
//!   be parsed at every call.

use cargo_metadata::MetadataCommand;
use once_cell::sync::Lazy;
use std::{
    collections::HashMap,
    env, fmt, fs,
    path::{Path, PathBuf},
};
use strum::{EnumCount, EnumIter, EnumString, EnumVariantNames, IntoEnumIterator};

/// `ethers_crate => name`
type CrateNames = HashMap<EthersCrate, &'static str>;

/// Maps an [`EthersCrate`] to its name in the compilation environment.
///
/// See [`determine_ethers_crates`] for more information.
static ETHERS_CRATE_NAMES: Lazy<CrateNames> = Lazy::new(determine_ethers_crates);

/// Returns the `core` crate's [`Path`][syn::Path].
#[inline]
pub fn ethers_core_crate() -> syn::Path {
    get_crate_path(EthersCrate::EthersCore)
}

/// Returns the `contract` crate's [`Path`][syn::Path].
#[inline]
pub fn ethers_contract_crate() -> syn::Path {
    get_crate_path(EthersCrate::EthersContract)
}

/// Returns the `providers` crate's [`Path`][syn::Path].
#[inline]
pub fn ethers_providers_crate() -> syn::Path {
    get_crate_path(EthersCrate::EthersProviders)
}

/// Returns an [`EthersCrate`]'s [`Path`][syn::Path] in the current project.
#[inline(always)]
pub fn get_crate_path(krate: EthersCrate) -> syn::Path {
    krate.get_path()
}

/// Determines the crate paths to use by looking at the [metadata][cargo_metadata] of the project.
///
/// The names will be:
/// - `ethers::*` if `ethers` is a dependency for all crates;
/// - for each `crate`:
///   - `ethers_<crate>` if it is a dependency, otherwise `ethers::<crate>`.
fn determine_ethers_crates() -> CrateNames {
    let default = || EthersCrate::ethers_path_names().collect();

    let manifest_dir: PathBuf = match env::var_os("CARGO_MANIFEST_DIR") {
        Some(s) => s.into(),
        None => return default(),
    };

    let lock_file = manifest_dir.join("Cargo.lock");
    let lock_file_existed = lock_file.exists();

    let names = crate_names_from_metadata(manifest_dir);

    // remove the lock file created from running the command
    if !lock_file_existed && lock_file.exists() {
        let _ = std::fs::remove_file(lock_file);
    }

    names.unwrap_or_else(default)
}

/// Runs [`cargo metadata`][MetadataCommand] from `manifest_dir` and determines the crate paths to
/// use.
///
/// Returns `None` on any error or if no dependencies are found.
#[inline]
fn crate_names_from_metadata(manifest_dir: PathBuf) -> Option<CrateNames> {
    let crate_is_root = is_crate_root(&manifest_dir);

    let metadata = MetadataCommand::new().current_dir(manifest_dir).exec().ok()?;
    let pkg = metadata.root_package()?;

    // return ethers_* if the root package is an EthersCrate (called in `ethers-rs/**/*`)
    if let Ok(current_pkg) = pkg.name.replace('_', "-").parse::<EthersCrate>() {
        // replace `current_pkg`'s name with "crate"
        let names = EthersCrate::path_names()
            .map(
                |(pkg, name)| {
                    if crate_is_root && pkg == current_pkg {
                        (pkg, "crate")
                    } else {
                        (pkg, name)
                    }
                },
            )
            .collect();
        return Some(names)
    } // else if pkg.name == "ethers" {} // should not happen: called in `ethers-rs/src/**/*`

    let mut names: CrateNames = EthersCrate::ethers_path_names().collect();
    for dep in pkg.dependencies.iter() {
        let name = dep.name.as_str();
        if let Ok(dep) = name.parse::<EthersCrate>() {
            names.insert(dep, dep.path_name());
        } else if name == "ethers" {
            return None
        }
    }
    Some(names)
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, EnumCount, EnumIter, EnumString, EnumVariantNames,
)]
#[strum(serialize_all = "kebab-case")]
pub enum EthersCrate {
    EthersAddressbook,
    EthersContract,
    EthersContractAbigen,
    EthersContractDerive,
    EthersCore,
    EthersDeriveEip712,
    EthersEtherscan,
    EthersMiddleware,
    EthersProviders,
    EthersSigners,
    EthersSolc,
}

impl AsRef<str> for EthersCrate {
    fn as_ref(&self) -> &str {
        self.crate_name()
    }
}

impl fmt::Display for EthersCrate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.pad(self.as_ref())
    }
}

impl EthersCrate {
    /// "`<self as kebab-case>`"
    #[inline]
    pub const fn crate_name(self) -> &'static str {
        match self {
            Self::EthersAddressbook => "ethers-addressbook",
            Self::EthersContract => "ethers-contract",
            Self::EthersContractAbigen => "ethers-contract-abigen",
            Self::EthersContractDerive => "ethers-contract-derive",
            Self::EthersCore => "ethers-core",
            Self::EthersDeriveEip712 => "ethers-derive-eip712",
            Self::EthersEtherscan => "ethers-etherscan",
            Self::EthersMiddleware => "ethers-middleware",
            Self::EthersProviders => "ethers-providers",
            Self::EthersSigners => "ethers-signers",
            Self::EthersSolc => "ethers-solc",
        }
    }

    /// "`<self as snake_case>`"
    #[inline]
    pub const fn path_name(self) -> &'static str {
        match self {
            Self::EthersAddressbook => "ethers_addressbook",
            Self::EthersContract => "ethers_contract",
            Self::EthersContractAbigen => "ethers_contract_abigen",
            Self::EthersContractDerive => "ethers_contract_derive",
            Self::EthersCore => "ethers_core",
            Self::EthersDeriveEip712 => "ethers_derive_eip712",
            Self::EthersEtherscan => "ethers_etherscan",
            Self::EthersMiddleware => "ethers_middleware",
            Self::EthersProviders => "ethers_providers",
            Self::EthersSigners => "ethers_signers",
            Self::EthersSolc => "ethers_solc",
        }
    }

    /// "ethers::`<self in ethers>`"
    #[inline]
    pub const fn ethers_path_name(self) -> &'static str {
        match self {
            Self::EthersAddressbook => "ethers::addressbook",
            Self::EthersContract => "ethers::contract",
            Self::EthersContractAbigen => "ethers::contract", // re-exported in ethers::contract
            Self::EthersContractDerive => "ethers::contract", // re-exported in ethers::contract
            Self::EthersCore => "ethers::core",
            Self::EthersDeriveEip712 => "ethers::contract", // re-exported in ethers::contract
            Self::EthersEtherscan => "ethers::etherscan",
            Self::EthersMiddleware => "ethers::middleware",
            Self::EthersProviders => "ethers::providers",
            Self::EthersSigners => "ethers::signers",
            Self::EthersSolc => "ethers::solc",
        }
    }

    /// `<ethers_*>`
    #[inline]
    pub fn path_names() -> impl Iterator<Item = (Self, &'static str)> {
        Self::iter().map(|x| (x, x.path_name()))
    }

    /// `<ethers::*>`
    #[inline]
    pub fn ethers_path_names() -> impl Iterator<Item = (Self, &'static str)> {
        Self::iter().map(|x| (x, x.ethers_path_name()))
    }

    /// Returns the [`Path`][syn::Path] in the current project.
    #[inline]
    pub fn get_path(&self) -> syn::Path {
        let name = ETHERS_CRATE_NAMES[self];
        syn::parse_str(name).unwrap()
    }
}

/// Returns whether `crate`, in the current environment, refers to the root package.
///
/// This is false for integration tests, benches, and examples, as the `crate` keyword will not
/// refer to the root package.
///
/// We can find this using some [environment variables set by Cargo during compilation][ref]:
/// - `CARGO_TARGET_TMPDIR` is only set when building integration test or benchmark code;
/// - When `CARGO_MANIFEST_DIR` contains `/benches/` or `/examples/`
/// - `CARGO_CRATE_NAME`, see [`is_crate_name_in_dirs`].
///
/// [ref]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
#[inline]
fn is_crate_root(manifest_dir: impl AsRef<Path>) -> bool {
    let manifest_dir = manifest_dir.as_ref();
    env::var_os("CARGO_TARGET_TMPDIR").is_none() &&
        manifest_dir.components().all(|c| {
            let s = c.as_os_str();
            s != "examples" && s != "benches"
        }) &&
        !is_crate_name_in_dirs(manifest_dir)
}

/// Returns whether `CARGO_CRATE_NAME` is the name of a file or directory in the first level of
/// `manifest_dir/{benches,examples,tests}/`.
///
/// # Example
///
/// With this project structure:
///
/// ```text
/// .
/// ├── Cargo.lock
/// ├── Cargo.toml
/// ├── src/
/// │   ...
/// ├── benches/
/// │   ├── large-input.rs
/// │   └── multi-file-bench/
/// │       ├── main.rs
/// │       └── bench_module.rs
/// ├── examples/
/// │   ├── simple.rs
/// │   └── multi-file-example/
/// │       ├── main.rs
/// │       └── ex_module.rs
/// └── tests/
///     ├── some-integration-tests.rs
///     └── multi-file-test/
///         ├── main.rs
///         └── test_module.rs
/// ```
///
/// The resulting `CARGO_CRATE_NAME` values will be:
///
/// |                  Path                  |          Value         |
/// |:-------------------------------------- | ----------------------:|
/// | benches/large-input.rs                 |            large-input |
/// | benches/multi-file-bench/\*\*/\*.rs    |       multi-file-bench |
/// | examples/simple.rs                     |                 simple |
/// | examples/multi-file-example/\*\*/\*.rs |     multi-file-example |
/// | tests/some-integration-tests.rs        | some-integration-tests |
/// | tests/multi-file-test/\*\*/\*.rs       |        multi-file-test |
#[inline]
fn is_crate_name_in_dirs(manifest_dir: &Path) -> bool {
    let crate_name = match env::var("CARGO_CRATE_NAME") {
        Ok(name) => name,
        Err(_) => return false,
    };

    let dirs =
        [manifest_dir.join("tests"), manifest_dir.join("examples"), manifest_dir.join("benches")];
    dirs.iter().any(|dir| {
        fs::read_dir(dir)
            .ok()
            .and_then(|entries| {
                entries.filter_map(Result::ok).find(|entry| file_stem_eq(entry.path(), &crate_name))
            })
            .is_some()
    })
}

#[inline]
fn file_stem_eq<T: AsRef<Path>, U: AsRef<str>>(path: T, s: U) -> bool {
    if let Some(stem) = path.as_ref().file_stem() {
        if let Some(stem) = stem.to_str() {
            return stem == s.as_ref()
        }
    }
    false
}
