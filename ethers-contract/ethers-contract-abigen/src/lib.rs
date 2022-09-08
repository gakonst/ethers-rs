#![deny(missing_docs, unsafe_code)]
#![deny(rustdoc::broken_intra_doc_links)]

//! Module for generating type-safe bindings to Ethereum smart contracts. This
//! module is intended to be used either indirectly with the `abigen` procedural
//! macro or directly from a build script / CLI

#[cfg(test)]
#[allow(missing_docs)]
#[macro_use]
#[path = "test/macros.rs"]
mod test_macros;

/// Contains types to generate rust bindings for solidity contracts
pub mod contract;
pub use contract::structs::InternalStructs;
use contract::Context;

pub mod rawabi;
pub use rawabi::RawAbi;

mod rustfmt;
mod source;
mod util;

pub mod filter;
pub use filter::{ContractFilter, ExcludeContracts, SelectContracts};
pub mod multi;
pub use multi::MultiAbigen;

pub use ethers_core::types::Address;
pub use source::Source;
pub use util::parse_address;

use crate::contract::ExpandedContract;
use eyre::Result;
use proc_macro2::TokenStream;
use std::{collections::HashMap, fs::File, io::Write, path::Path};

/// Builder struct for generating type-safe bindings from a contract's ABI
///
/// Note: Your contract's ABI must contain the `stateMutability` field. This is
/// [still not supported by Vyper](https://github.com/vyperlang/vyper/issues/1931), so you must adjust your ABIs and replace
/// `constant` functions with `view` or `pure`.
///
/// To generate bindings for _multiple_ contracts at once see also [`crate::MultiAbigen`].
///
/// # Example
///
/// Running the code below will generate a file called `token.rs` containing the
/// bindings inside, which exports an `ERC20Token` struct, along with all its events. Put into a
/// `build.rs` file this will generate the bindings during `cargo build`.
///
/// ```no_run
/// # use ethers_contract_abigen::Abigen;
/// # fn foo() -> Result<(), Box<dyn std::error::Error>> {
/// Abigen::new("ERC20Token", "./abi.json")?.generate()?.write_to_file("token.rs")?;
/// # Ok(())
/// # }
#[derive(Debug, Clone)]
pub struct Abigen {
    /// The source of the ABI JSON for the contract whose bindings
    /// are being generated.
    abi_source: Source,

    /// Override the contract name to use for the generated type.
    contract_name: String,

    /// Manually specified contract method aliases.
    method_aliases: HashMap<String, String>,

    /// Derives added to event structs and enums.
    event_derives: Vec<String>,

    /// Format the code using a locally installed copy of `rustfmt`.
    rustfmt: bool,

    /// Manually specified event name aliases.
    event_aliases: HashMap<String, String>,

    /// Manually specified error name aliases.
    error_aliases: HashMap<String, String>,
}

impl Abigen {
    /// Creates a new builder with the given ABI JSON source.
    pub fn new<S: AsRef<str>>(contract_name: &str, abi_source: S) -> Result<Self> {
        let abi_source = abi_source.as_ref().parse()?;
        Ok(Self {
            abi_source,
            contract_name: contract_name.to_owned(),
            method_aliases: HashMap::new(),
            event_derives: Vec::new(),
            event_aliases: HashMap::new(),
            rustfmt: true,
            error_aliases: Default::default(),
        })
    }

    /// Attempts to load a new builder from an ABI JSON file at the specific
    /// path.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let name = path
            .as_ref()
            .file_stem()
            .ok_or_else(|| eyre::format_err!("Missing file stem in path"))?
            .to_str()
            .ok_or_else(|| eyre::format_err!("Unable to convert file stem to string"))?;

        Self::new(name, std::fs::read_to_string(path.as_ref())?)
    }

    /// Manually adds a solidity event alias to specify what the event struct
    /// and function name will be in Rust.
    #[must_use]
    pub fn add_event_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.event_aliases.insert(signature.into(), alias.into());
        self
    }

    /// Manually adds a solidity method alias to specify what the method name
    /// will be in Rust. For solidity methods without an alias, the snake cased
    /// method name will be used.
    #[must_use]
    pub fn add_method_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.method_aliases.insert(signature.into(), alias.into());
        self
    }

    /// Manually adds a solidity error alias to specify what the error struct will be in Rust.
    #[must_use]
    pub fn add_error_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.error_aliases.insert(signature.into(), alias.into());
        self
    }

    /// Specify whether or not to format the code using a locally installed copy
    /// of `rustfmt`.
    ///
    /// Note that in case `rustfmt` does not exist or produces an error, the
    /// unformatted code will be used.
    #[must_use]
    pub fn rustfmt(mut self, rustfmt: bool) -> Self {
        self.rustfmt = rustfmt;
        self
    }

    /// Add a custom derive to the derives for event structs and enums.
    ///
    /// This makes it possible to for example derive serde::Serialize and
    /// serde::Deserialize for events.
    #[must_use]
    pub fn add_event_derive<S>(mut self, derive: S) -> Self
    where
        S: Into<String>,
    {
        self.event_derives.push(derive.into());
        self
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings> {
        let rustfmt = self.rustfmt;
        let name = self.contract_name.clone();
        let (expanded, _) = self.expand()?;
        Ok(ContractBindings { tokens: expanded.into_tokens(), rustfmt, name })
    }

    /// Expands the `Abigen` and returns the [`ExpandedContract`] that holds all tokens and the
    /// [`Context`] that holds the state used during expansion.
    pub fn expand(self) -> Result<(ExpandedContract, Context)> {
        let ctx = Context::from_abigen(self)?;
        Ok((ctx.expand()?, ctx))
    }
}

/// Type-safe contract bindings generated by a `Builder`. This type can be
/// either written to file or into a token stream for use in a procedural macro.
pub struct ContractBindings {
    /// The TokenStream representing the contract bindings.
    tokens: TokenStream,
    /// The output options used for serialization.
    rustfmt: bool,
    /// The contract name
    name: String,
}

impl ContractBindings {
    /// Writes the bindings to a given `Write`.
    pub fn write<W>(&self, mut w: W) -> Result<()>
    where
        W: Write,
    {
        let source = {
            let raw = self.tokens.to_string();

            if self.rustfmt {
                rustfmt::format(&raw).unwrap_or(raw)
            } else {
                raw
            }
        };

        w.write_all(source.as_bytes())?;
        Ok(())
    }

    /// Writes the bindings to a new Vec. Panics if unable to allocate
    pub fn to_vec(&self) -> Vec<u8> {
        let mut bindings = vec![];
        self.write(&mut bindings).expect("allocations don't fail");
        bindings
    }

    /// Writes the bindings to the specified file.
    pub fn write_to_file<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(path)?;
        self.write(file)
    }

    /// Writes the bindings to a `contract_name.rs` file in the specified
    /// directory. The filename is the snake_case transformation of the contract
    /// name.
    pub fn write_module_in_dir<P>(&self, dir: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = dir.as_ref().join(self.module_filename());
        self.write_to_file(file)
    }

    /// Converts the bindings into its underlying token stream. This allows it
    /// to be used within a procedural macro.
    pub fn into_tokens(self) -> TokenStream {
        self.tokens
    }

    /// Generate the default module name (snake case of the contract name)
    pub fn module_name(&self) -> String {
        util::safe_module_name(&self.name)
    }

    /// Generate the default filename of the module
    pub fn module_filename(&self) -> String {
        let mut name = self.module_name();
        name.extend([".rs"]);
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
}
