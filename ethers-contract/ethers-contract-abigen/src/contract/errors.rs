//! derive error bindings

use super::{util, Context};
use crate::contract::common::{expand_data_struct, expand_data_tuple, expand_params};
use ethers_core::{
    abi::{ethabi::AbiError, ErrorExt},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use eyre::Result;
use inflector::Inflector;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

impl Context {
    /// Returns all error declarations
    pub(crate) fn errors(&self) -> Result<TokenStream> {
        let data_types = self
            .abi
            .errors
            .values()
            .flatten()
            .map(|event| self.expand_error(event))
            .collect::<Result<Vec<_>>>()?;

        // only expand an enum when multiple errors are present
        let errors_enum_decl = if self.abi.errors.values().flatten().count() > 1 {
            self.expand_errors_enum()
        } else {
            quote! {}
        };

        Ok(quote! {
            // HERE
            #( #data_types )*

            #errors_enum_decl

            // HERE end
        })
    }

    /// Expands an ABI error into a single error data type. This can expand either
    /// into a structure or a tuple in the case where all error parameters are anonymous.
    fn expand_error(&self, error: &AbiError) -> Result<TokenStream> {
        let sig = self.error_aliases.get(&error.abi_signature()).cloned();
        let abi_signature = error.abi_signature();

        let error_name = error_struct_name(&error.name, sig);

        let fields = self.expand_error_params(error)?;

        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = error.inputs.iter().all(|input| input.name.is_empty());
        let data_type_definition = if all_anonymous_fields {
            // expand to a tuple struct
            expand_data_tuple(&error_name, &fields)
        } else {
            // expand to a struct
            expand_data_struct(&error_name, &fields)
        };

        let doc = format!(
            "Custom Error type `{}` with signature `{}` and selector `{:?}`",
            error.name,
            abi_signature,
            error.selector()
        );
        let abi_signature_doc = util::expand_doc(&doc);
        let ethers_contract = ethers_contract_crate();
        // use the same derives as for events
        let derives = util::expand_derives(&self.event_derives);

        let error_name = &error.name;

        Ok(quote! {
             #abi_signature_doc
            #[derive(Clone, Debug, Default, Eq, PartialEq, #ethers_contract::EthError, #ethers_contract::EthDisplay, #derives)]
            #[etherror( name = #error_name, abi = #abi_signature )]
            pub #data_type_definition
        })
    }

    /// Expands to the `name : type` pairs of the function's outputs
    fn expand_error_params(&self, error: &AbiError) -> Result<Vec<(TokenStream, TokenStream)>> {
        expand_params(&error.inputs, |s| self.internal_structs.get_struct_type(s))
    }

    /// The name ident of the errors enum
    fn expand_error_enum_name(&self) -> Ident {
        util::ident(&format!("{}Errors", self.contract_ident))
    }

    /// Generate an enum with a variant for each event
    fn expand_errors_enum(&self) -> TokenStream {
        let variants = self
            .abi
            .errors
            .values()
            .flatten()
            .map(|err| {
                error_struct_name(&err.name, self.error_aliases.get(&err.abi_signature()).cloned())
            })
            .collect::<Vec<_>>();

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        // use the same derives as for events
        let derives = util::expand_derives(&self.event_derives);
        let enum_name = self.expand_error_enum_name();

        quote! {
           #[derive(Debug, Clone, PartialEq, Eq, #ethers_contract::EthAbiType, #derives)]
            pub enum #enum_name {
                #(#variants(#variants)),*
            }

        impl  #ethers_core::abi::AbiDecode for #enum_name {
            fn decode(data: impl AsRef<[u8]>) -> ::std::result::Result<Self, #ethers_core::abi::AbiError> {
                 #(
                    if let Ok(decoded) = <#variants as #ethers_core::abi::AbiDecode>::decode(data.as_ref()) {
                        return Ok(#enum_name::#variants(decoded))
                    }
                )*
                Err(#ethers_core::abi::Error::InvalidData.into())
            }
        }

         impl  #ethers_core::abi::AbiEncode for #enum_name {
            fn encode(self) -> Vec<u8> {
                match self {
                    #(
                        #enum_name::#variants(element) => element.encode()
                    ),*
                }
            }
        }

        impl ::std::fmt::Display for #enum_name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    #(
                        #enum_name::#variants(element) => element.fmt(f)
                    ),*
                }
            }
        }

        #(
            impl ::std::convert::From<#variants> for #enum_name {
                fn from(var: #variants) -> Self {
                    #enum_name::#variants(var)
                }
            }
        )*

        }
    }
}

/// Expands an ABI error into an identifier for its event data type.
fn error_struct_name(error_name: &str, alias: Option<Ident>) -> Ident {
    alias.unwrap_or_else(|| util::ident(error_name))
}

/// Returns the alias name for an error
pub(crate) fn error_struct_alias(name: &str) -> Ident {
    util::ident(&name.to_pascal_case())
}
