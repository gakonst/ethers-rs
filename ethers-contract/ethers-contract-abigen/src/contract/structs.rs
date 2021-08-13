//! Methods for expanding structs
use anyhow::{Context as _, Result};
use inflector::Inflector;
use proc_macro2::{Literal, TokenStream};
use quote::quote;

use ethers_core::abi::{struct_def::FieldType, ParamType};

use crate::contract::{types, Context};
use crate::util;
use ethers_core::abi::struct_def::StructFieldType;

impl Context {
    /// Generate corresponding types for structs parsed from a human readable ABI
    ///
    /// NOTE: This assumes that all structs that are potentially used as type for variable are
    /// in fact present in the `AbiParser`, this is sound because `AbiParser::parse` would have
    /// failed already
    pub fn abi_structs(&self) -> Result<TokenStream> {
        let mut structs = TokenStream::new();
        for (name, sol_struct) in &self.abi_parser.structs {
            let mut fields = Vec::with_capacity(sol_struct.fields().len());
            let mut param_types = Vec::with_capacity(sol_struct.fields().len());
            for field in sol_struct.fields() {
                let field_name = util::ident(&field.name().to_snake_case());
                match field.r#type() {
                    FieldType::Elementary(ty) => {
                        param_types.push(ty.clone());
                        let ty = types::expand(ty)?;
                        fields.push(quote! { pub #field_name: #ty });
                    }
                    FieldType::Struct(struct_ty) => {
                        let ty = expand_struct_type(struct_ty);
                        fields.push(quote! { pub #field_name: #ty });

                        let name = struct_ty.name();
                        let tuple = self
                            .abi_parser
                            .struct_tuples
                            .get(name)
                            .context(format!("No types found for {}", name))?
                            .clone();
                        let tuple = ParamType::Tuple(tuple);

                        param_types.push(struct_ty.as_param(tuple));
                    }
                    FieldType::Mapping(_) => {
                        return Err(anyhow::anyhow!(
                            "Mapping types in struct `{}` are not supported {:?}",
                            name,
                            field
                        ));
                    }
                }
            }

            let abi_signature = format!(
                "{}({})",
                name,
                param_types
                    .iter()
                    .map(|kind| kind.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            );

            let abi_signature_doc = util::expand_doc(&format!("`{}`", abi_signature));

            let name = util::ident(name);

            // use the same derives as for events
            let derives = &self.event_derives;
            let derives = quote! {#(#derives),*};

            structs.extend(quote! {
                #abi_signature_doc
                #[derive(Clone, Debug, Default, Eq, PartialEq, ethers::contract::EthAbiType, #derives)]
                pub struct #name {
                    #( #fields ),*
                }
            });
        }
        Ok(structs)
    }
}

/// Expands to the rust struct type
fn expand_struct_type(struct_ty: &StructFieldType) -> TokenStream {
    match struct_ty {
        StructFieldType::Type(ty) => {
            let ty = util::ident(ty.name());
            quote! {#ty}
        }
        StructFieldType::Array(ty) => {
            let ty = expand_struct_type(&*ty);
            quote! {::std::vec::Vec<#ty>}
        }
        StructFieldType::FixedArray(ty, size) => {
            let ty = expand_struct_type(&*ty);
            quote! { [#ty; #size]}
        }
    }
}