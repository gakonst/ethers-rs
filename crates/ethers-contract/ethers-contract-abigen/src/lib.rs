#![allow(dead_code)]
#![allow(unused_imports)]
#![deny(missing_docs, unsafe_code)]

//! Module for generating type-safe bindings to Ethereum smart contracts. This
//! module is intended to be used either indirectly with the `abigen` procedural
//! macro or directly from a build script / CLI

#[cfg(test)]
#[allow(missing_docs)]
#[macro_use]
#[path = "test/macros.rs"]
mod test_macros;

mod contract;
use contract::Context;

mod rustfmt;
mod source;
mod util;

pub use ethers_types::Address;
pub use source::Source;
pub use util::parse_address;

use anyhow::Result;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Internal global arguments passed to the generators for each individual
/// component that control expansion.
pub(crate) struct Args {
    /// The source of the ABI JSON for the contract whose bindings
    /// are being generated.
    abi_source: Source,

    /// Override the contract name to use for the generated type.
    contract_name: String,

    /// The runtime crate name to use.
    runtime_crate_name: String,

    /// The visibility modifier to use for the generated module and contract
    /// re-export.
    visibility_modifier: Option<String>,

    /// Override the contract module name that contains the generated code.
    contract_mod_override: Option<String>,

    /// Manually specified contract method aliases.
    method_aliases: HashMap<String, String>,

    /// Derives added to event structs and enums.
    event_derives: Vec<String>,
}

impl Args {
    /// Creates a new builder given the path to a contract's truffle artifact
    /// JSON file.
    pub fn new(contract_name: &str, abi_source: Source) -> Self {
        Args {
            abi_source,
            contract_name: contract_name.to_owned(),

            runtime_crate_name: "abigen".to_owned(),
            visibility_modifier: None,
            contract_mod_override: None,
            method_aliases: HashMap::new(),
            event_derives: Vec::new(),
        }
    }
}

/// Internal output options for controlling how the generated code gets
/// serialized to file.
struct SerializationOptions {
    /// Format the code using a locally installed copy of `rustfmt`.
    rustfmt: bool,
}

impl Default for SerializationOptions {
    fn default() -> Self {
        SerializationOptions { rustfmt: true }
    }
}

/// Builder for generating contract code. Note that no code is generated until
/// the builder is finalized with `generate` or `output`.
pub struct Builder {
    /// The contract binding generation args.
    args: Args,
    /// The serialization options.
    options: SerializationOptions,
}

impl Builder {
    /// Creates a new builder given the contract's ABI JSON string
    pub fn from_str(name: &str, abi: &str) -> Self {
        Builder::source(name, Source::String(abi.to_owned()))
    }

    /// Creates a new builder given the path to a contract's ABI file
    pub fn new<P>(name: &str, path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Builder::source(name, Source::local(path))
    }

    /// Creates a new builder from a source URL.
    pub fn from_url<S>(name: &str, url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let source = Source::parse(url)?;
        Ok(Builder::source(name, source))
    }

    /// Creates a new builder with the given ABI JSON source.
    pub fn source(name: &str, source: Source) -> Self {
        Builder {
            args: Args::new(name, source),
            options: SerializationOptions::default(),
        }
    }

    /// Sets the crate name for the runtime crate. This setting is usually only
    /// needed if the crate was renamed in the Cargo manifest.
    pub fn runtime_crate_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.args.runtime_crate_name = name.into();
        self
    }

    /// Sets an optional visibility modifier for the generated module and
    /// contract re-export.
    pub fn visibility_modifier<S>(mut self, vis: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.args.visibility_modifier = vis.map(S::into);
        self
    }

    /// Sets the optional contract module name override.
    pub fn contract_mod_override<S>(mut self, name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.args.contract_mod_override = name.map(S::into);
        self
    }

    /// Manually adds a solidity method alias to specify what the method name
    /// will be in Rust. For solidity methods without an alias, the snake cased
    /// method name will be used.
    pub fn add_method_alias<S1, S2>(mut self, signature: S1, alias: S2) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        self.args
            .method_aliases
            .insert(signature.into(), alias.into());
        self
    }

    /// Specify whether or not to format the code using a locally installed copy
    /// of `rustfmt`.
    ///
    /// Note that in case `rustfmt` does not exist or produces an error, the
    /// unformatted code will be used.
    pub fn rustfmt(mut self, rustfmt: bool) -> Self {
        self.options.rustfmt = rustfmt;
        self
    }

    /// Add a custom derive to the derives for event structs and enums.
    ///
    /// This makes it possible to for example derive serde::Serialize and
    /// serde::Deserialize for events.
    pub fn add_event_derive<S>(mut self, derive: S) -> Self
    where
        S: Into<String>,
    {
        self.args.event_derives.push(derive.into());
        self
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings> {
        let tokens = Context::expand(self.args)?;
        Ok(ContractBindings {
            tokens,
            options: self.options,
        })
    }
}

/// Type-safe contract bindings generated by a `Builder`. This type can be
/// either written to file or into a token stream for use in a procedural macro.
pub struct ContractBindings {
    /// The TokenStream representing the contract bindings.
    tokens: TokenStream,
    /// The output options used for serialization.
    options: SerializationOptions,
}

impl ContractBindings {
    /// Writes the bindings to a given `Write`.
    pub fn write<W>(&self, mut w: W) -> Result<()>
    where
        W: Write,
    {
        let source = {
            let raw = self.tokens.to_string();

            if self.options.rustfmt {
                rustfmt::format(&raw).unwrap_or(raw)
            } else {
                raw
            }
        };

        w.write_all(source.as_bytes())?;
        Ok(())
    }

    /// Writes the bindings to the specified file.
    pub fn write_to_file<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = File::create(path)?;
        self.write(file)
    }

    /// Converts the bindings into its underlying token stream. This allows it
    /// to be used within a procedural macro.
    pub fn into_tokens(self) -> TokenStream {
        self.tokens
    }
}
