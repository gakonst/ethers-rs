//! TODO

use eyre::Result;
use inflector::Inflector;
use std::{collections::BTreeMap, fs, io::Write, path::Path};

use crate::{util, Abigen, ContractBindings};

/// Collects Abigen structs for a series of contracts, pending generation of
/// the contract bindings.
#[derive(Debug, Clone)]
pub struct MultiAbigen {
    /// Abigen objects to be written
    abigens: Vec<Abigen>,
}

impl std::ops::Deref for MultiAbigen {
    type Target = Vec<Abigen>;

    fn deref(&self) -> &Self::Target {
        &self.abigens
    }
}

impl From<Vec<Abigen>> for MultiAbigen {
    fn from(abigens: Vec<Abigen>) -> Self {
        Self { abigens }
    }
}

impl std::iter::FromIterator<Abigen> for MultiAbigen {
    fn from_iter<I: IntoIterator<Item = Abigen>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl MultiAbigen {
    /// Create a new instance from a series (`contract name`, `abi_source`)
    ///
    /// See `Abigen::new`
    pub fn new<I, Name, Source>(abis: I) -> Result<Self>
    where
        I: IntoIterator<Item = (Name, Source)>,
        Name: AsRef<str>,
        Source: AsRef<str>,
    {
        let abis = abis
            .into_iter()
            .map(|(contract_name, abi_source)| Abigen::new(contract_name.as_ref(), abi_source))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self::from_abigens(abis))
    }

    /// Create a new instance from a series of already resolved `Abigen`
    pub fn from_abigens(abis: impl IntoIterator<Item = Abigen>) -> Self {
        abis.into_iter().collect()
    }

    /// Reads all json files contained in the given `dir` and use the file name for the name of the
    /// `ContractBindings`.
    /// This is equivalent to calling `MultiAbigen::new` with all the json files and their filename.
    ///
    /// # Example
    ///
    /// ```text
    /// abi
    /// ├── ERC20.json
    /// ├── Contract1.json
    /// ├── Contract2.json
    /// ...
    /// ```
    ///
    /// ```no_run
    /// # use ethers_contract_abigen::MultiAbigen;
    /// let gen = MultiAbigen::from_json_files("./abi").unwrap();
    /// ```
    pub fn from_json_files(root: impl AsRef<Path>) -> Result<Self> {
        util::json_files(root.as_ref()).into_iter().map(Abigen::from_file).collect()
    }

    /// Add another Abigen to the module or lib
    pub fn push(&mut self, abigen: Abigen) {
        self.abigens.push(abigen)
    }

    /// Build the contract bindings and prepare for writing
    pub fn build(self) -> Result<MultiBindings> {
        let bindings = self
            .abigens
            .into_iter()
            .map(|v| v.generate())
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(|v| (v.name.clone(), v))
            .collect();

        Ok(MultiBindings { bindings })
    }
}

/// Output of the [`MultiAbigen`] build process. `MultiBindings` wraps a group
/// of built contract bindings that have yet to be written to disk.
///
/// `MultiBindings` enables the user to
/// 1. Write a collection of bindings to a rust module
/// 2. Write a collection of bindings to a rust lib
/// 3. Ensure that a collection of bindings matches an on-disk module or lib.
///
/// Generally we recommend writing the bindings to a module folder within your
/// rust project. Users seeking to create "official" bindings for some project
/// may instead write an entire library to publish via crates.io.
///
/// Rather than using `MultiAbigen` in a build script, we recommend committing
/// the generated files, and replacing the build script with an integration
/// test. To enable this, we have provided
/// `MultiBindings::ensure_consistent_bindings` and
/// `MultiBindings::ensure_consistent_crate`. These functions generate the
/// expected module or library in memory, and check that the on-disk files
/// match the expected files. We recommend running these inside CI.
///
/// This has several advantages:
///   * No need for downstream users to compile the build script
///   * No need for downstream users to run the whole `abigen!` generation steps
///   * The generated code is more usable in an IDE
///   * CI will fail if the generated code is out of date (if `abigen!` or the contract's ABI itself
///     changed)
pub struct MultiBindings {
    /// Abigen objects to be written
    bindings: BTreeMap<String, ContractBindings>,
}

// deref allows for inspection without modification
impl std::ops::Deref for MultiBindings {
    type Target = BTreeMap<String, ContractBindings>;

    fn deref(&self) -> &Self::Target {
        &self.bindings
    }
}

impl MultiBindings {
    /// Generat the contents of the `Cargo.toml` file for a lib
    fn generate_cargo_toml(
        &self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<Vec<u8>> {
        let mut toml = vec![];

        writeln!(toml, "[package]")?;
        writeln!(toml, r#"name = "{}""#, name.as_ref())?;
        writeln!(toml, r#"version = "{}""#, version.as_ref())?;
        writeln!(toml, r#"edition = "2021""#)?;
        writeln!(toml)?;
        writeln!(toml, "# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html")?;
        writeln!(toml)?;
        writeln!(toml, "[dependencies]")?;
        writeln!(
            toml,
            r#"
ethers = {{ git = "https://github.com/gakonst/ethers-rs", default-features = false }}
serde_json = "1.0.79"
"#
        )?;
        Ok(toml)
    }

    /// Write the contents of `Cargo.toml` to disk
    fn write_cargo_toml(
        &self,
        lib: &Path,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
    ) -> Result<()> {
        let contents = self.generate_cargo_toml(name, version)?;

        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(lib.join("Cargo.toml"))?;
        file.write_all(&contents)?;

        Ok(())
    }

    /// Generate the shared prefix of the `lib.rs` or `mod.rs`
    fn generate_prefix(
        &self,
        mut buf: impl Write,
        is_crate: bool,
        single_file: bool,
    ) -> Result<()> {
        writeln!(buf, "#![allow(clippy::all)]")?;
        writeln!(
            buf,
            "//! This {} contains abigen! generated bindings for solidity contracts.",
            if is_crate { "lib" } else { "module" }
        )?;
        writeln!(buf, "//! This is autogenerated code.")?;
        writeln!(buf, "//! Do not manually edit these files.")?;
        writeln!(
            buf,
            "//! {} may be overwritten by the codegen system at any time.",
            if single_file && !is_crate { "This file" } else { "These files" }
        )?;

        Ok(())
    }

    /// Append module declarations to the `lib.rs` or `mod.rs`
    fn append_module_names(&self, mut buf: impl Write) -> Result<()> {
        // sorting here not necessary, as btreemap keys are ordered
        for module in self.bindings.keys().map(|name| format!("pub mod {};", name.to_snake_case()))
        {
            writeln!(buf, "{}", module)?;
        }

        Ok(())
    }

    /// Generate the contents of `lib.rs` or `mod.rs`
    fn generate_super_contents(&self, is_crate: bool, single_file: bool) -> Result<Vec<u8>> {
        let mut contents = vec![];
        self.generate_prefix(&mut contents, is_crate, single_file)?;

        if !single_file {
            self.append_module_names(&mut contents)?;
        } else {
            for binding in self.bindings.values() {
                binding.write(&mut contents)?;
            }
        }

        Ok(contents)
    }

    /// Write the `lib.rs` or `mod.rs` to disk
    fn write_super_file(&self, path: &Path, is_crate: bool, single_file: bool) -> Result<()> {
        let filename = if is_crate { "lib.rs" } else { "mod.rs" };
        let contents = self.generate_super_contents(is_crate, single_file)?;
        fs::write(path.join(filename), contents)?;
        Ok(())
    }

    /// Write all contract bindings to their respective files
    fn write_bindings(&self, path: &Path) -> Result<()> {
        for binding in self.bindings.values() {
            binding.write_module_in_dir(path)?;
        }
        Ok(())
    }

    /// Generates all the bindings and writes them to the given module
    ///
    /// # Example
    ///
    /// Read all json abi files from the `./abi` directory
    /// ```text
    /// abi
    /// ├── ERC20.json
    /// ├── Contract1.json
    /// ├── Contract2.json
    /// ...
    /// ```
    ///
    /// and write them to the `./src/contracts` location as
    ///
    /// ```text
    /// src/contracts
    /// ├── mod.rs
    /// ├── er20.rs
    /// ├── contract1.rs
    /// ├── contract2.rs
    /// ...
    /// ```
    ///
    /// ```no_run
    /// # use ethers_contract_abigen::MultiAbigen;
    /// let gen = MultiAbigen::from_json_files("./abi").unwrap();
    /// let bindings = gen.build().unwrap();
    /// bindings.write_to_module("./src/contracts", false).unwrap();
    /// ```
    pub fn write_to_module(self, module: impl AsRef<Path>, single_file: bool) -> Result<()> {
        let module = module.as_ref();
        fs::create_dir_all(module)?;

        self.write_super_file(module, false, single_file)?;

        if !single_file {
            self.write_bindings(module)?;
        }
        Ok(())
    }

    /// Generates all the bindings and writes a library crate containing them
    /// to the provided path
    ///
    /// # Example
    ///
    /// Read all json abi files from the `./abi` directory
    /// ```text
    /// abi
    /// ├── ERC20.json
    /// ├── Contract1.json
    /// ├── Contract2.json
    /// ├── Contract3/
    ///     ├── Contract3.json
    /// ...
    /// ```
    ///
    /// and write them to the `./bindings` location as
    ///
    /// ```text
    /// bindings
    /// ├── Cargo.toml
    /// ├── src/
    ///     ├── lib.rs
    ///     ├── er20.rs
    ///     ├── contract1.rs
    ///     ├── contract2.rs
    /// ...
    /// ```
    ///
    /// ```no_run
    /// # use ethers_contract_abigen::MultiAbigen;
    /// let gen = MultiAbigen::from_json_files("./abi").unwrap();
    /// let bindings = gen.build().unwrap();
    /// bindings.write_to_crate(
    ///     "my-crate", "0.0.5", "./bindings", false
    /// ).unwrap();
    /// ```
    pub fn write_to_crate(
        self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
        lib: impl AsRef<Path>,
        single_file: bool,
    ) -> Result<()> {
        let lib = lib.as_ref();
        let src = lib.join("src");
        fs::create_dir_all(&src)?;

        self.write_cargo_toml(lib, name, version)?;
        self.write_super_file(&src, true, single_file)?;

        if !single_file {
            self.write_bindings(&src)?;
        }

        Ok(())
    }

    /// Ensures the contents of the bindings directory are correct
    ///
    /// Does this by first generating the `lib.rs` or `mod.rs`, then the
    /// contents of each binding file in turn.
    fn ensure_consistent_bindings(
        self,
        dir: impl AsRef<Path>,
        is_crate: bool,
        single_file: bool,
    ) -> Result<()> {
        let dir = dir.as_ref();
        let super_name = if is_crate { "lib.rs" } else { "mod.rs" };

        let super_contents = self.generate_super_contents(is_crate, single_file)?;
        check_file_in_dir(dir, super_name, &super_contents)?;

        // If it is single file, we skip checking anything but the super
        // contents
        if !single_file {
            for binding in self.bindings.values() {
                check_binding_in_dir(dir, binding)?;
            }
        }

        Ok(())
    }

    /// This ensures that the already generated bindings crate matches the
    /// output of a fresh new run. Run this in a rust test, to get notified in
    /// CI if the newly generated bindings deviate from the already generated
    /// ones, and it's time to generate them again. This could happen if the
    /// ABI of a contract or the output that `ethers` generates changed.
    ///
    /// If this functions is run within a test during CI and fails, then it's
    /// time to update all bindings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the freshly generated bindings match with the
    /// existing bindings. Otherwise an `Err(_)` containing an `eyre::Report`
    /// with more information.
    ///
    /// # Example
    ///
    /// Check that the generated files are up to date
    ///
    /// ```no_run
    /// # use ethers_contract_abigen::MultiAbigen;
    /// #[test]
    /// fn generated_bindings_are_fresh() {
    ///  let project_root = std::path::Path::new(&env!("CARGO_MANIFEST_DIR"));
    ///  let abi_dir = project_root.join("abi");
    ///  let gen = MultiAbigen::from_json_files(&abi_dir).unwrap();
    ///  let bindings = gen.build().unwrap();
    ///  bindings.ensure_consistent_crate(
    ///     "my-crate", "0.0.1", project_root.join("src/contracts"), false
    ///  ).expect("inconsistent bindings");
    /// }
    /// ```
    pub fn ensure_consistent_crate(
        self,
        name: impl AsRef<str>,
        version: impl AsRef<str>,
        crate_path: impl AsRef<Path>,
        single_file: bool,
    ) -> Result<()> {
        let crate_path = crate_path.as_ref();

        // additionally check the contents of the cargo
        let cargo_contents = self.generate_cargo_toml(name, version)?;
        check_file_in_dir(crate_path, "Cargo.toml", &cargo_contents)?;

        self.ensure_consistent_bindings(crate_path.join("src"), true, single_file)?;
        Ok(())
    }

    /// This ensures that the already generated bindings module matches the
    /// output of a fresh new run. Run this in a rust test, to get notified in
    /// CI if the newly generated bindings deviate from the already generated
    /// ones, and it's time to generate them again. This could happen if the
    /// ABI of a contract or the output that `ethers` generates changed.
    ///
    /// If this functions is run within a test during CI and fails, then it's
    /// time to update all bindings.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the freshly generated bindings match with the
    /// existing bindings. Otherwise an `Err(_)` containing an `eyre::Report`
    /// with more information.
    ///
    /// # Example
    ///
    /// Check that the generated files are up to date
    ///
    /// ```no_run
    /// # use ethers_contract_abigen::MultiAbigen;
    /// #[test]
    /// fn generated_bindings_are_fresh() {
    ///  let project_root = std::path::Path::new(&env!("CARGO_MANIFEST_DIR"));
    ///  let abi_dir = project_root.join("abi");
    ///  let gen = MultiAbigen::from_json_files(&abi_dir).unwrap();
    ///  let bindings = gen.build().unwrap();
    ///  bindings.ensure_consistent_module(
    ///     project_root.join("src/contracts"), false
    ///  ).expect("inconsistent bindings");
    /// }
    /// ```
    pub fn ensure_consistent_module(
        self,
        module: impl AsRef<Path>,
        single_file: bool,
    ) -> Result<()> {
        self.ensure_consistent_bindings(module, false, single_file)?;
        Ok(())
    }
}

fn check_file_in_dir(dir: &Path, file_name: &str, expected_contents: &[u8]) -> Result<()> {
    eyre::ensure!(dir.is_dir(), "Not a directory: {}", dir.display());

    let file_path = dir.join(file_name);
    eyre::ensure!(file_path.is_file(), "Not a file: {}", file_path.display());

    let contents = fs::read(file_path).expect("Unable to read file");
    eyre::ensure!(contents == expected_contents, "file contents do not match");
    Ok(())
}

fn check_binding_in_dir(dir: &Path, binding: &ContractBindings) -> Result<()> {
    let name = binding.module_filename();
    let contents = binding.to_vec();

    check_file_in_dir(dir, &name, &contents)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{panic, path::PathBuf};

    struct Context {
        multi_gen: MultiAbigen,
        mod_root: PathBuf,
    }

    fn run_test<T>(test: T)
    where
        T: FnOnce(&Context) + panic::UnwindSafe,
    {
        let crate_root = std::path::Path::new(&env!("CARGO_MANIFEST_DIR")).to_owned();
        let console = Abigen::new(
            "Console",
            crate_root.join("../tests/solidity-contracts/console.json").display().to_string(),
        )
        .unwrap();

        let simple_storage = Abigen::new(
            "SimpleStorage",
            crate_root
                .join("../tests/solidity-contracts/simplestorage_abi.json")
                .display()
                .to_string(),
        )
        .unwrap();

        let human_readable = Abigen::new(
            "HrContract",
            r#"[
        struct Foo { uint256 x; }
        function foo(Foo memory x)
        function bar(uint256 x, uint256 y, address addr)
        yeet(uint256,uint256,address)
    ]"#,
        )
        .unwrap();

        let multi_gen = MultiAbigen::from_abigens([console, simple_storage, human_readable]);

        let mod_root = tempfile::tempdir().unwrap().path().join("contracts");
        let context = Context { multi_gen, mod_root };

        let result = panic::catch_unwind(|| test(&context));

        assert!(result.is_ok())
    }

    #[test]
    fn can_generate_multi_file_module() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = false;

            multi_gen.clone().build().unwrap().write_to_module(&mod_root, single_file).unwrap();
            multi_gen
                .clone()
                .build()
                .unwrap()
                .ensure_consistent_module(&mod_root, single_file)
                .expect("Inconsistent bindings");
        })
    }

    #[test]
    fn can_generate_single_file_module() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = true;

            multi_gen.clone().build().unwrap().write_to_module(&mod_root, single_file).unwrap();
            multi_gen
                .clone()
                .build()
                .unwrap()
                .ensure_consistent_module(&mod_root, single_file)
                .expect("Inconsistent bindings");
        })
    }

    #[test]
    fn can_generate_multi_file_crate() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = false;
            let name = "a-name";
            let version = "290.3782.3";

            multi_gen
                .clone()
                .build()
                .unwrap()
                .write_to_crate(name, version, &mod_root, single_file)
                .unwrap();
            multi_gen
                .clone()
                .build()
                .unwrap()
                .ensure_consistent_crate(name, version, &mod_root, single_file)
                .expect("Inconsistent bindings");
        })
    }

    #[test]
    fn can_generate_single_file_crate() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = true;
            let name = "a-name";
            let version = "290.3782.3";

            multi_gen
                .clone()
                .build()
                .unwrap()
                .write_to_crate(name, version, &mod_root, single_file)
                .unwrap();
            multi_gen
                .clone()
                .build()
                .unwrap()
                .ensure_consistent_crate(name, version, &mod_root, single_file)
                .expect("Inconsistent bindings");
        })
    }

    #[test]
    fn can_detect_incosistent_multi_file_module() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = false;

            multi_gen.clone().build().unwrap().write_to_module(&mod_root, single_file).unwrap();

            let mut cloned = multi_gen.clone();
            cloned.push(
                Abigen::new(
                    "AdditionalContract",
                    r#"[
                        getValue() (uint256)
                    ]"#,
                )
                .unwrap(),
            );

            let result =
                cloned.build().unwrap().ensure_consistent_module(&mod_root, single_file).is_err();

            // ensure inconsistent bindings are detected
            assert!(result, "Inconsistent bindings wrongly approved");
        })
    }

    #[test]
    fn can_detect_incosistent_single_file_module() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = true;

            multi_gen.clone().build().unwrap().write_to_module(&mod_root, single_file).unwrap();

            let mut cloned = multi_gen.clone();
            cloned.push(
                Abigen::new(
                    "AdditionalContract",
                    r#"[
                        getValue() (uint256)
                    ]"#,
                )
                .unwrap(),
            );

            let result =
                cloned.build().unwrap().ensure_consistent_module(&mod_root, single_file).is_err();

            // ensure inconsistent bindings are detected
            assert!(result, "Inconsistent bindings wrongly approved");
        })
    }

    #[test]
    fn can_detect_incosistent_multi_file_crate() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = false;
            let name = "a-name";
            let version = "290.3782.3";

            multi_gen
                .clone()
                .build()
                .unwrap()
                .write_to_crate(name, version, &mod_root, single_file)
                .unwrap();

            let mut cloned = multi_gen.clone();
            cloned.push(
                Abigen::new(
                    "AdditionalContract",
                    r#"[
                            getValue() (uint256)
                        ]"#,
                )
                .unwrap(),
            );

            let result = cloned
                .build()
                .unwrap()
                .ensure_consistent_crate(name, version, &mod_root, single_file)
                .is_err();

            // ensure inconsistent bindings are detected
            assert!(result, "Inconsistent bindings wrongly approved");
        })
    }

    #[test]
    fn can_detect_incosistent_single_file_crate() {
        run_test(|context| {
            let Context { multi_gen, mod_root } = context;

            let single_file = true;
            let name = "a-name";
            let version = "290.3782.3";

            multi_gen
                .clone()
                .build()
                .unwrap()
                .write_to_crate(name, version, &mod_root, single_file)
                .unwrap();

            let mut cloned = multi_gen.clone();
            cloned.push(
                Abigen::new(
                    "AdditionalContract",
                    r#"[
                            getValue() (uint256)
                        ]"#,
                )
                .unwrap(),
            );

            let result = cloned
                .build()
                .unwrap()
                .ensure_consistent_crate(name, version, &mod_root, single_file)
                .is_err();

            // ensure inconsistent bindings are detected
            assert!(result, "Inconsistent bindings wrongly approved");
        })
    }
}
