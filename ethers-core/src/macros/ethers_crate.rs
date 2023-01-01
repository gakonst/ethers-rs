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

#[cfg(test)]
use rand::{
    distributions::{Distribution, Standard},
    Rng,
};

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

    // return ethers_* if the root package is an internal ethers crate since `ethers` is not
    // available
    if let Ok(current_pkg) = pkg.name.parse::<EthersCrate>() {
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
    } /* else if pkg.name == "ethers" {
          // should not happen (the root package the `ethers` workspace package itself)
      } */

    let mut names: CrateNames = EthersCrate::ethers_path_names().collect();
    for dep in pkg.dependencies.iter() {
        let name = dep.name.as_str();
        if name.starts_with("ethers") {
            if name == "ethers" {
                return None
            } else if let Ok(dep) = name.parse::<EthersCrate>() {
                names.insert(dep, dep.path_name());
            }
        }
    }
    Some(names)
}

/// An `ethers-rs` workspace crate.
#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    EnumCount,
    EnumIter,
    EnumString,
    EnumVariantNames,
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

#[cfg(test)]
impl Distribution<EthersCrate> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> EthersCrate {
        const RANGE: std::ops::Range<u8> = 0..EthersCrate::COUNT as u8;
        // SAFETY: generates in the safe range
        unsafe { std::mem::transmute(rng.gen_range(RANGE)) }
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

    /// The path on the file system, from an `ethers-rs` root directory.
    #[inline]
    pub const fn fs_path(self) -> &'static str {
        match self {
            Self::EthersContractAbigen => "ethers-contract/ethers-contract-abigen",
            Self::EthersContractDerive => "ethers-contract/ethers-contract-derive",
            Self::EthersDeriveEip712 => "ethers-core/ethers-derive-eip712",
            _ => self.crate_name(),
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

    let dirs = ["tests", "examples", "benches"].map(|d| manifest_dir.join(d));
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};
    use std::{
        collections::{BTreeMap, HashSet},
        env,
        ffi::OsStr,
        fs,
        process::Command,
    };
    use tempfile::TempDir;

    const DIRS: &[&str] = &["benches", "examples", "tests"];

    #[test]
    fn test_names() {
        fn assert_names(
            dir: &TempDir,
            crate_name: &str,
            sub_name: &str,
            ethers: bool,
            dependencies: &[EthersCrate],
        ) {
            let root = dir.path();
            with_test_manifest(root, crate_name, ethers, dependencies);

            let names: CrateNames = determine_ethers_crates(root, sub_name);

            let krate = crate_name.parse::<EthersCrate>();
            let is_internal = krate.is_ok();
            let mut expected: CrateNames = match (is_internal, ethers) {
                // internal
                (true, _) => EthersCrate::path_names().collect(),

                // ethers
                (_, true) => EthersCrate::ethers_path_names().collect(),

                // no ethers
                (_, false) => {
                    let mut n: CrateNames = EthersCrate::ethers_path_names().collect();
                    for &dep in dependencies {
                        n.insert(dep, dep.path_name());
                    }
                    n
                }
            };

            if is_internal && crate_name == sub_name {
                expected.insert(krate.unwrap(), "crate");
            }

            // don't use assert for a better custom message
            if names != expected {
                // BTreeMap sorts the keys
                let names: BTreeMap<_, _> = names.into_iter().collect();
                let expected: BTreeMap<_, _> = expected.into_iter().collect();
                panic!("\nCase failed: (`{crate_name}`, `{sub_name}`, `{ethers}`, `{dependencies:?}`)\nNames: {names:#?}\nExpected: {expected:#?}\n");
            }
        }

        fn gen_unique<const N: usize>() -> [EthersCrate; N] {
            assert!(N < EthersCrate::COUNT);
            let rng = &mut thread_rng();
            let mut set = HashSet::with_capacity(N);
            while set.len() < N {
                set.insert(rng.gen());
            }
            let vec: Vec<_> = set.into_iter().collect();
            vec.try_into().unwrap()
        }

        let dir = test_project();
        let crate_name = dir.path().file_name().unwrap().to_str().unwrap();
        for name in [crate_name, "ethers-contract"] {
            // only ethers
            assert_names(&dir, name, name, true, &[]);

            // only others
            assert_names(&dir, name, name, false, gen_unique::<3>().as_slice());

            // ethers and others
            assert_names(&dir, name, name, true, gen_unique::<3>().as_slice());
        }
    }

    #[test]
    fn test_is_crate_root() {
        let dir = test_project();
        let root = dir.path();

        assert!(is_crate_root(root, root.file_name().unwrap()));

        // `CARGO_TARGET_TMPDIR`
        // name, path name or path validity not checked
        env::set_var("CARGO_TARGET_TMPDIR", root.join("target/tmp"));
        assert!(!is_crate_root(root, "simple_tests"));
        assert!(!is_crate_root(root, "complex_tests"));
        assert!(!is_crate_root(root, "simple_benches"));
        assert!(!is_crate_root(root, "complex_benches"));
        assert!(!is_crate_root(root, "non_existant"));
        assert!(!is_crate_root(root.join("does-not-exist"), "foo_bar"));
        env::remove_var("CARGO_TARGET_TMPDIR");

        // `CARGO_TARGET_TMPDIR`
        // complex path has `/{dir_name}/` in the path
        // name or path validity not checked
        assert!(!is_crate_root(root.join("examples/complex_examples"), "complex-examples"));
        assert!(!is_crate_root(root.join("benches/complex_benches"), "complex-benches"));
    }

    #[test]
    fn test_is_crate_in_dirs() {
        let dir = test_project();
        let root = dir.path();

        for dir_name in DIRS {
            assert!(is_crate_name_in_dirs(root, format!("simple_{dir_name}")));
            assert!(is_crate_name_in_dirs(root, format!("complex_{dir_name}")));
        }

        assert!(!is_crate_name_in_dirs(root, "non_existant"));
        assert!(!is_crate_name_in_dirs(root.join("does-not-exist"), "foo_bar"));
    }

    #[test]
    fn test_file_stem_eq() {
        let path = Path::new("/tmp/foo.rs");
        assert!(file_stem_eq(path, "foo"));
        assert!(!file_stem_eq(path, "tmp"));
        assert!(!file_stem_eq(path, "foo.rs"));
        assert!(!file_stem_eq(path, "fo"));
        assert!(!file_stem_eq(path, "f"));
        assert!(!file_stem_eq(path, ""));

        let path = Path::new("/tmp/foo/");
        assert!(file_stem_eq(path, "foo"));
        assert!(!file_stem_eq(path, "tmp"));
        assert!(!file_stem_eq(path, "fo"));
        assert!(!file_stem_eq(path, "f"));
        assert!(!file_stem_eq(path, ""));
    }

    // utils

    /// Creates:
    ///
    /// ```text
    /// - new_dir
    ///   - src
    ///     - main.rs
    ///   - {dir_name} for dir_name in DIRS
    ///     - simple_{dir_name}.rs
    ///     - complex_{dir_name}
    ///       - src if not "tests"
    ///         - main.rs
    ///         - module.rs
    /// ```
    fn test_project() -> TempDir {
        // without the default `.` which is not a valid crate name
        let dir = tempfile::Builder::new().prefix("tmp").tempdir().unwrap();
        let root = dir.path();
        env::set_current_dir(root).unwrap();
        let _ = Command::new("cargo").arg("init").current_dir(root).output().unwrap();

        for &dir_name in DIRS {
            let new_dir = root.join(dir_name);
            fs::create_dir_all(&new_dir).unwrap();

            let simple = new_dir.join(format!("simple_{dir_name}.rs"));
            fs::write(simple, "").unwrap();

            let mut complex = new_dir.join(format!("complex_{dir_name}"));
            if dir_name != "tests" {
                fs::create_dir(&complex).unwrap();
                fs::write(complex.join("Cargo.toml"), "").unwrap();
                complex.push("src");
            }
            fs::create_dir(&complex).unwrap();
            fs::write(complex.join("main.rs"), "").unwrap();
            fs::write(complex.join("module.rs"), "").unwrap();
        }

        // create target dirs
        let target = root.join("target");
        fs::create_dir(&target).unwrap();
        fs::create_dir_all(target.join("tmp")).unwrap();

        dir
    }

    /// Writes a test manifest to `{root}/Cargo.toml`.
    fn with_test_manifest(
        root: impl AsRef<Path>,
        name: &str,
        ethers: bool,
        dependencies: &[EthersCrate],
    ) {
        // use paths to avoid downloading dependencies
        const ETHERS_CORE: &str = env!("CARGO_MANIFEST_DIR");
        let ethers_root = Path::new(ETHERS_CORE).parent().unwrap();
        let mut dependencies_toml =
            String::with_capacity(150 * (ethers as usize + dependencies.len()));

        if ethers {
            let path = escaped_path(ethers_root);
            let ethers = format!("ethers = {{ path = \"{path}\" }}\n");
            dependencies_toml.push_str(&ethers);
        }

        for dep in dependencies.iter() {
            let path = escaped_path(ethers_root.join(dep.fs_path()));
            let dep = format!("{dep} = {{ path = \"{path}\" }}\n");
            dependencies_toml.push_str(&dep);
        }

        let contents = format!(
            r#"
[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
{dependencies_toml}
"#
        );
        fs::write(root.as_ref().join("Cargo.toml"), contents).unwrap();
    }

    fn escaped_path(path: impl AsRef<Path>) -> impl std::fmt::Display {
        path.as_ref().display().to_string().replace(r"\", r"\\")
    }

    // wrappers for overriding env
    fn determine_ethers_crates(
        manifest_dir: impl AsRef<OsStr>,
        crate_name: impl AsRef<OsStr>,
    ) -> CrateNames {
        run_with_env(
            &[
                ("CARGO_MANIFEST_DIR", manifest_dir.as_ref()),
                ("CARGO_CRATE_NAME", crate_name.as_ref()),
            ],
            super::determine_ethers_crates,
        )
    }

    fn is_crate_root(manifest_dir: impl AsRef<Path>, crate_name: impl AsRef<OsStr>) -> bool {
        run_with_env(&[("CARGO_CRATE_NAME", crate_name)], || {
            super::is_crate_root(manifest_dir.as_ref())
        })
    }

    fn is_crate_name_in_dirs(
        manifest_dir: impl AsRef<Path>,
        crate_name: impl AsRef<OsStr>,
    ) -> bool {
        run_with_env(&[("CARGO_CRATE_NAME", crate_name)], || {
            super::is_crate_name_in_dirs(manifest_dir.as_ref())
        })
    }

    fn run_with_env<F, T, K, V>(env: &[(K, V)], f: F) -> T
    where
        F: Fn() -> T,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        let mut previous = Vec::with_capacity(env.len());
        for (key, value) in env.iter() {
            // store old
            previous.push((key, env::var_os(value)));
            // set new
            env::set_var(key, value);
        }

        // run
        let res = f();

        // set old
        for (key, value) in previous {
            match value {
                Some(value) => env::set_var(key, value),
                None => env::remove_var(key),
            }
        }

        res
    }
}
