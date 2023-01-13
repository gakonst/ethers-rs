//! # Abigen
//!
//! Programmatically generate type-safe Rust bindings for Ethereum smart contracts.
//!
//! This crate is intended to be used either indirectly with the [`abigen` procedural macro][abigen]
//! or directly from a build script / CLI.
//!
//! [abigen]: https://docs.rs/ethers/latest/ethers/contract/macro.abigen.html

#![deny(rustdoc::broken_intra_doc_links, missing_docs, unsafe_code)]

#[cfg(test)]
#[allow(missing_docs)]
#[macro_use]
#[path = "test/macros.rs"]
mod test_macros;

pub mod contract;
pub use contract::structs::InternalStructs;

pub mod filter;
pub use filter::{ContractFilter, ExcludeContracts, SelectContracts};

pub mod multi;
pub use multi::MultiAbigen;

mod source;
pub use source::Source;

pub mod util;

pub use ethers_core::types::Address;

use contract::{Context, ExpandedContract};
use eyre::{Context as _, Result};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::{collections::HashMap, fmt, fs, io, path::Path};

/// Programmatically generate type-safe Rust bindings for an Ethereum smart contract from its ABI.
///
/// For all the supported ABI sources, see [Source].
///
/// To generate bindings for *multiple* contracts at once, see [`MultiAbigen`].
///
/// To generate bindings at compile time, see [the abigen! macro][abigen], or use in a `build.rs`
/// file.
///
/// [abigen]: https://docs.rs/ethers/latest/ethers/contract/macro.abigen.html
///
/// # Example
///
/// Running the code below will generate a file called `token.rs` containing the bindings inside,
/// which exports an `ERC20Token` struct, along with all its events.
///
/// ```no_run
/// # use ethers_contract_abigen::Abigen;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// Abigen::new("ERC20Token", "./abi.json")?.generate()?.write_to_file("token.rs")?;
/// # Ok(())
/// # }
#[derive(Clone, Debug)]
#[must_use = "Abigen does nothing unless you generate or expand it."]
pub struct Abigen {
    /// The source of the ABI JSON for the contract whose bindings are being generated.
    abi_source: Source,

    /// The contract's name to use for the generated type.
    contract_name: String,

    /// Manually specified contract method aliases.
    method_aliases: HashMap<String, String>,

    /// Manually specified `derive` macros added to all structs and enums.
    derives: Vec<String>,

    /// Whether to format the generated bindings using [`prettyplease`].
    format: bool,

    /// Manually specified event name aliases.
    event_aliases: HashMap<String, String>,

    /// Manually specified error name aliases.
    error_aliases: HashMap<String, String>,
}

impl Abigen {
    /// Creates a new builder with the given [ABI Source][Source].
    pub fn new<T: Into<String>, S: AsRef<str>>(contract_name: T, abi_source: S) -> Result<Self> {
        let abi_source = abi_source.as_ref().parse()?;
        Ok(Self {
            abi_source,
            contract_name: contract_name.into(),
            format: true,
            method_aliases: Default::default(),
            derives: Default::default(),
            event_aliases: Default::default(),
            error_aliases: Default::default(),
        })
    }

    /// Attempts to load a new builder from an ABI JSON file at the specific path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = dunce::canonicalize(path).wrap_err("File does not exist")?;
        // this shouldn't error when the path is canonicalized
        let file_name = path.file_name().ok_or_else(|| eyre::eyre!("Invalid path"))?;
        let name = file_name
            .to_str()
            .ok_or_else(|| eyre::eyre!("File name contains invalid UTF-8"))?
            .split('.') // ignore everything after the first `.`
            .next()
            .unwrap(); // file_name is not empty as asserted by .file_name() already
        let contents = fs::read_to_string(&path).wrap_err("Could not read file")?;

        Self::new(name, contents)
    }

    /// Manually adds a solidity event alias to specify what the event struct and function name will
    /// be in Rust.
    ///
    /// For events without an alias, the `PascalCase` event name will be used.
    pub fn add_event_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.event_aliases.insert(signature.into(), alias.into());
        self
    }

    /// Add a Solidity method error alias to specify the generated method name.
    ///
    /// For methods without an alias, the `snake_case` method name will be used.
    pub fn add_method_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.method_aliases.insert(signature.into(), alias.into());
        self
    }

    /// Add a Solidity custom error alias to specify the generated struct's name.
    ///
    /// For errors without an alias, the `PascalCase` error name will be used.
    pub fn add_error_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.error_aliases.insert(signature.into(), alias.into());
        self
    }

    #[must_use]
    #[deprecated = "Use format instead"]
    #[doc(hidden)]
    pub fn rustfmt(mut self, rustfmt: bool) -> Self {
        self.format = rustfmt;
        self
    }

    /// Specify whether to format the code or not. True by default.
    ///
    /// This will use [`prettyplease`], so the resulting formatted code **will not** be affected by
    /// the local `rustfmt` version or config.
    pub fn format(mut self, format: bool) -> Self {
        self.format = format;
        self
    }

    #[deprecated = "Use add_derive instead"]
    #[doc(hidden)]
    pub fn add_event_derive<S: Into<String>>(mut self, derive: S) -> Self {
        self.derives.push(derive.into());
        self
    }

    /// Add a custom derive to the derives for all structs and enums.
    ///
    /// For example, this makes it possible to derive serde::Serialize and serde::Deserialize.
    pub fn add_derive<S: Into<String>>(mut self, derive: S) -> Self {
        self.derives.push(derive.into());
        self
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings> {
        let format = self.format;
        let name = self.contract_name.clone();
        let (expanded, _) = self.expand()?;
        Ok(ContractBindings { tokens: expanded.into_tokens(), format, name })
    }

    /// Expands the `Abigen` and returns the [`ExpandedContract`] that holds all tokens and the
    /// [`Context`] that holds the state used during expansion.
    pub fn expand(self) -> Result<(ExpandedContract, Context)> {
        let ctx = Context::from_abigen(self)?;
        Ok((ctx.expand()?, ctx))
    }
}

/// Type-safe contract bindings generated by `Abigen`.
///
/// This type can be either written to file or converted to a token stream for a procedural macro.
#[derive(Clone)]
pub struct ContractBindings {
    /// The contract's name.
    pub name: String,

    /// The generated bindings as a `TokenStream`.
    pub tokens: TokenStream,

    /// Whether to format the generated bindings using [`prettyplease`].
    pub format: bool,
}

impl ToTokens for ContractBindings {
    fn into_token_stream(self) -> TokenStream {
        self.tokens
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(Some(self.tokens.clone()))
    }

    fn to_token_stream(&self) -> TokenStream {
        self.tokens.clone()
    }
}

impl fmt::Display for ContractBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.format {
            let syntax_tree = syn::parse2::<syn::File>(self.tokens.clone()).unwrap();
            let s = prettyplease::unparse(&syntax_tree);
            f.write_str(&s)
        } else {
            fmt::Display::fmt(&self.tokens, f)
        }
    }
}

impl fmt::Debug for ContractBindings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ContractBindings")
            .field("name", &self.name)
            .field("format", &self.format)
            .finish()
    }
}

impl ContractBindings {
    /// Writes the bindings to a new Vec.
    pub fn to_vec(&self) -> Vec<u8> {
        self.to_string().into_bytes()
    }

    /// Writes the bindings to a given `io::Write`.
    pub fn write(&self, w: &mut impl io::Write) -> io::Result<()> {
        let tokens = self.to_string();
        w.write_all(tokens.as_bytes())
    }

    /// Writes the bindings to a given `fmt::Write`.
    pub fn write_fmt(&self, w: &mut impl fmt::Write) -> fmt::Result {
        let tokens = self.to_string();
        w.write_str(&tokens)
    }

    /// Writes the bindings to the specified file.
    pub fn write_to_file(&self, file: impl AsRef<Path>) -> io::Result<()> {
        fs::write(file.as_ref(), self.to_string())
    }

    /// Writes the bindings to a `contract_name.rs` file in the specified directory.
    pub fn write_module_in_dir(&self, dir: impl AsRef<Path>) -> io::Result<()> {
        let file = dir.as_ref().join(self.module_filename());
        self.write_to_file(file)
    }

    #[deprecated = "Use ::quote::ToTokens::into_token_stream instead"]
    #[doc(hidden)]
    pub fn into_tokens(self) -> TokenStream {
        self.tokens
    }

    /// Generate the default module name (snake case of the contract name).
    pub fn module_name(&self) -> String {
        util::safe_module_name(&self.name)
    }

    /// Generate the default file name of the module.
    pub fn module_filename(&self) -> String {
        let mut name = self.module_name();
        name.push_str(".rs");
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ethers_solc::project_util::TempProject;

    #[test]
    fn can_generate_structs() {
        let greeter = include_str!("../../tests/solidity-contracts/greeter_with_struct.json");
        let abigen = Abigen::new("Greeter", greeter).unwrap();
        let gen = abigen.generate().unwrap();
        let out = gen.tokens.to_string();
        assert!(out.contains("pub struct Stuff"));
    }

    #[test]
    fn can_compile_and_generate() {
        let tmp = TempProject::dapptools().unwrap();

        tmp.add_source(
            "Greeter",
            r#"
// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0;

contract Greeter {

    struct Inner {
        bool a;
    }

    struct Stuff {
        Inner inner;
    }

    function greet(Stuff calldata stuff) public view returns (Stuff memory) {
        return stuff;
    }
}
"#,
        )
        .unwrap();

        let _ = tmp.compile().unwrap();

        let abigen =
            Abigen::from_file(tmp.artifacts_path().join("Greeter.sol/Greeter.json")).unwrap();
        let gen = abigen.generate().unwrap();
        let out = gen.tokens.to_string();
        assert!(out.contains("pub struct Stuff"));
        assert!(out.contains("pub struct Inner"));
    }

    #[test]
    fn can_compile_and_generate_with_punctuation() {
        let tmp = TempProject::dapptools().unwrap();

        tmp.add_source(
            "Greeter.t.sol",
            r#"
// SPDX-License-Identifier: MIT
pragma solidity >=0.8.0;

contract Greeter {
    struct Inner {
        bool a;
    }
    struct Stuff {
        Inner inner;
    }
    function greet(Stuff calldata stuff) public view returns (Stuff memory) {
        return stuff;
    }
}
"#,
        )
        .unwrap();

        let _ = tmp.compile().unwrap();

        let abigen =
            Abigen::from_file(tmp.artifacts_path().join("Greeter.t.sol/Greeter.json")).unwrap();
        let gen = abigen.generate().unwrap();
        let out = gen.tokens.to_string();
        assert!(out.contains("pub struct Stuff"));
        assert!(out.contains("pub struct Inner"));
    }
}
