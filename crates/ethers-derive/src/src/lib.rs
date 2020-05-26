#![deny(missing_docs, unsafe_code)]

//! Crate for generating type-safe bindings to Ethereum smart contracts. This
//! crate is intended to be used either indirectly with the `ethcontract`
//! crate's `contract` procedural macro or directly from a build script.

#[cfg(test)]
#[allow(missing_docs)]
#[macro_use]
#[path = "test/macros.rs"]
mod test_macros;

mod contract;
mod rustfmt;
mod source;
mod util;

pub use crate::source::Source;
pub use crate::util::parse_address;
use anyhow::Result;
pub use ethcontract_common::Address;
use proc_macro2::TokenStream;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Internal global arguments passed to the generators for each individual
/// component that control expansion.
pub(crate) struct Args {
    /// The source of the truffle artifact JSON for the contract whose bindings
    /// are being generated.
    artifact_source: Source,
    /// The runtime crate name to use.
    runtime_crate_name: String,
    /// The visibility modifier to use for the generated module and contract
    /// re-export.
    visibility_modifier: Option<String>,
    /// Override the contract module name that contains the generated code.
    contract_mod_override: Option<String>,
    /// Override the contract name to use for the generated type.
    contract_name_override: Option<String>,
    /// Manually specified deployed contract addresses.
    deployments: HashMap<u32, Address>,
    /// Manually specified contract method aliases.
    method_aliases: HashMap<String, String>,
    /// Derives added to event structs and enums.
    event_derives: Vec<String>,
}

impl Args {
    /// Creates a new builder given the path to a contract's truffle artifact
    /// JSON file.
    pub fn new(source: Source) -> Self {
        Args {
            artifact_source: source,
            runtime_crate_name: "ethcontract".to_owned(),
            visibility_modifier: None,
            contract_mod_override: None,
            contract_name_override: None,
            deployments: HashMap::new(),
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
    /// Creates a new builder given the path to a contract's truffle artifact
    /// JSON file.
    pub fn new<P>(artifact_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Builder::with_source(Source::local(artifact_path))
    }

    /// Creates a new builder from a source URL.
    pub fn from_source_url<S>(source_url: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let source = Source::parse(source_url)?;
        Ok(Builder::with_source(source))
    }

    /// Creates a new builder with the given artifact JSON source.
    pub fn with_source(source: Source) -> Self {
        Builder {
            args: Args::new(source),
            options: SerializationOptions::default(),
        }
    }

    /// Sets the crate name for the runtime crate. This setting is usually only
    /// needed if the crate was renamed in the Cargo manifest.
    pub fn with_runtime_crate_name<S>(mut self, name: S) -> Self
    where
        S: Into<String>,
    {
        self.args.runtime_crate_name = name.into();
        self
    }

    /// Sets an optional visibility modifier for the generated module and
    /// contract re-export.
    pub fn with_visibility_modifier<S>(mut self, vis: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.args.visibility_modifier = vis.map(S::into);
        self
    }

    /// Sets the optional contract module name override.
    pub fn with_contract_mod_override<S>(mut self, name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.args.contract_mod_override = name.map(S::into);
        self
    }

    /// Sets the optional contract name override. This setting is needed when
    /// using a artifact JSON source that does not provide a contract name such
    /// as Etherscan.
    pub fn with_contract_name_override<S>(mut self, name: Option<S>) -> Self
    where
        S: Into<String>,
    {
        self.args.contract_name_override = name.map(S::into);
        self
    }

    /// Manually adds specifies the deployed address of a contract for a given
    /// network. Note that manually specified deployments take precedence over
    /// deployments in the Truffle artifact (in the `networks` property of the
    /// artifact).
    ///
    /// This is useful for integration test scenarios where the address of a
    /// contract on the test node is deterministic (for example using
    /// `ganache-cli -d`) but the contract address is not part of the Truffle
    /// artifact; or to override a deployment included in a Truffle artifact.
    pub fn add_deployment(mut self, network_id: u32, address: Address) -> Self {
        self.args.deployments.insert(network_id, address);
        self
    }

    /// Manually adds specifies the deployed address as a string of a contract
    /// for a given network. See `Builder::add_deployment` for more information.
    ///
    /// # Panics
    ///
    /// This method panics if the specified address string is invalid. See
    /// `parse_address` for more information on the address string format.
    pub fn add_deployment_str<S>(self, network_id: u32, address: S) -> Self
    where
        S: AsRef<str>,
    {
        self.add_deployment(
            network_id,
            parse_address(address).expect("failed to parse address"),
        )
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
    pub fn with_rustfmt(mut self, rustfmt: bool) -> Self {
        self.options.rustfmt = rustfmt;
        self
    }

    /// Add a custom derive to the derives for event structs and enums.
    ///
    /// This makes it possible to for example derive serde::Serialize and
    /// serde::Deserialize for events.
    ///
    /// # Examples
    ///
    /// ```
    /// use ethcontract_generate::Builder;
    /// let builder = Builder::new("path")
    ///     .add_event_derive("serde::Serialize")
    ///     .add_event_derive("serde::Deserialize");
    /// ```
    pub fn add_event_derive<S>(mut self, derive: S) -> Self
    where
        S: Into<String>,
    {
        self.args.event_derives.push(derive.into());
        self
    }

    /// Generates the contract bindings.
    pub fn generate(self) -> Result<ContractBindings> {
        let tokens = contract::expand(self.args)?;
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
