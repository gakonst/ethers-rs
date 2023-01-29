//! Custom errors expansion

use super::{
    common::{expand_params, expand_struct},
    util, Context,
};
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
            #( #data_types )*

            #errors_enum_decl
        })
    }

    /// Expands an ABI error into a single error data type. This can expand either
    /// into a structure or a tuple in the case where all error parameters are anonymous.
    fn expand_error(&self, error: &AbiError) -> Result<TokenStream> {
        let error_name = &error.name;
        let abi_signature = error.abi_signature();

        let alias_opt = self.error_aliases.get(&abi_signature).cloned();
        let error_struct_name = error_struct_name(&error.name, alias_opt);

        let fields = self.expand_error_params(error)?;

        // expand as a tuple if all fields are anonymous
        let all_anonymous_fields = error.inputs.iter().all(|input| input.name.is_empty());
        let data_type_definition = expand_struct(&error_struct_name, &fields, all_anonymous_fields);

        let doc_str = format!(
            "Custom Error type `{error_name}` with signature `{abi_signature}` and selector `0x{}`",
            hex::encode(error.selector())
        );

        let mut extra_derives = self.expand_extra_derives();
        if util::can_derive_defaults(&error.inputs) {
            extra_derives.extend(quote!(Default));
        }

        let ethers_contract = ethers_contract_crate();

        Ok(quote! {
            #[doc = #doc_str]
            #[derive(Clone, Debug, Eq, PartialEq, #ethers_contract::EthError, #ethers_contract::EthDisplay, #extra_derives)]
            #[etherror(name = #error_name, abi = #abi_signature)]
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
        let enum_name = self.expand_error_enum_name();
        let variants = self
            .abi
            .errors
            .values()
            .flatten()
            .map(|err| {
                error_struct_name(&err.name, self.error_aliases.get(&err.abi_signature()).cloned())
            })
            .collect::<Vec<_>>();

        let extra_derives = self.expand_extra_derives();

        let ethers_core = ethers_core_crate();
        let ethers_contract = ethers_contract_crate();

        quote! {
            #[doc = "Container type for all of the contract's custom errors"]
            #[derive(Debug, Clone, PartialEq, Eq, #ethers_contract::EthAbiType, #extra_derives)]
            pub enum #enum_name {
                #( #variants(#variants), )*
            }

            impl #ethers_core::abi::AbiDecode for #enum_name {
                fn decode(data: impl AsRef<[u8]>) -> ::core::result::Result<Self, #ethers_core::abi::AbiError> {
                    let data = data.as_ref();
                    #(
                        if let Ok(decoded) = <#variants as #ethers_core::abi::AbiDecode>::decode(data) {
                            return Ok(Self::#variants(decoded))
                        }
                    )*
                    Err(#ethers_core::abi::Error::InvalidData.into())
                }
            }

            impl #ethers_core::abi::AbiEncode for #enum_name {
                fn encode(self) -> ::std::vec::Vec<u8> {
                    match self {
                        #(
                            Self::#variants(element) => #ethers_core::abi::AbiEncode::encode(element),
                        )*
                    }
                }
            }

            impl ::core::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    match self {
                        #(
                            Self::#variants(element) => ::core::fmt::Display::fmt(element, f)
                        ),*
                    }
                }
            }

            #(
                impl ::core::convert::From<#variants> for #enum_name {
                    fn from(value: #variants) -> Self {
                        Self::#variants(value)
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
