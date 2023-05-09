//! Helper functions for deriving `EthCall`

use crate::{calllike::*, utils, utils::ident};
use ethers_core::{
    abi::{FunctionExt, HumanReadableParser},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Error, DeriveInput};

/// Generates the `ethcall` trait support
pub(crate) fn derive_eth_call_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let attributes = parse_calllike_attributes!(input, "ethcall");

    let function_call_name = attributes.name(&input.ident);
    let mut function = if let Some((abi, span)) = attributes.abi() {
        let sig = abi.trim_start_matches("function ").trim_start();
        // try to parse as solidity function
        match HumanReadableParser::parse_function(&abi) {
            Ok(fun) => fun,
            Err(parse_err) => {
                return derive_trait_impls_with_abi_type(&input, &function_call_name, Some(sig))
                    .map_err(|e| {
                        let mut error = Error::new(span, parse_err);
                        error.combine(Error::new(span, e));
                        error
                    })
            }
        }
    } else {
        // try to determine the abi by using its fields at runtime
        return derive_trait_impls_with_abi_type(&input, &function_call_name, None)
    };
    function.name = function_call_name.clone();

    let sig = function.abi_signature();
    let selector = utils::selector(function.selector());
    let decode_impl = derive_decode_impl_from_params(&function.inputs, ident("EthCall"));

    derive_trait_impls(
        &input,
        &function_call_name,
        quote!(#sig.into()),
        Some(selector),
        decode_impl,
    )
}

/// Use the `AbiType` trait to determine the correct `ParamType` and signature at runtime
fn derive_trait_impls_with_abi_type(
    input: &DeriveInput,
    function_call_name: &str,
    abi_signature: Option<&str>,
) -> Result<TokenStream, Error> {
    let mut abi_signature = if let Some(sig) = abi_signature {
        quote!(#sig)
    } else {
        utils::abi_signature_with_abi_type(input, function_call_name, "EthCall")?
    };
    abi_signature.extend(quote!(.into()));
    let decode_impl = derive_decode_impl_with_abi_type(input, ident("EthCall"))?;
    derive_trait_impls(input, function_call_name, abi_signature, None, decode_impl)
}

/// Generates the EthCall implementation
pub fn derive_trait_impls(
    input: &DeriveInput,
    function_call_name: &str,
    abi_signature: TokenStream,
    selector: Option<TokenStream>,
    decode_impl: TokenStream,
) -> Result<TokenStream, Error> {
    // the ethers crates to use
    let ethers_core = ethers_core_crate();
    let ethers_contract = ethers_contract_crate();
    let struct_name = &input.ident;

    let selector = selector.unwrap_or_else(|| {
        quote! {
            #ethers_core::utils::id(Self::abi_signature())
        }
    });

    let ethcall_impl = quote! {
        impl #ethers_contract::EthCall for #struct_name {
            fn function_name() -> ::std::borrow::Cow<'static, str> {
                #function_call_name.into()
            }

            fn selector() -> #ethers_core::types::Selector {
                #selector
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi_signature
            }
        }
    };
    let codec_impl = derive_codec_impls(input, decode_impl, ident("EthCall"))?;

    Ok(quote! {
        #ethcall_impl
        #codec_impl
    })
}
