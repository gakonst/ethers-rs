//! Helper functions for deriving `EthError`

use crate::{calllike::*, utils, utils::ident};
use ethers_core::{
    abi::{ErrorExt, HumanReadableParser},
    macros::{ethers_contract_crate, ethers_core_crate},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Error, DeriveInput};

/// Generates the `EthError` trait support
pub(crate) fn derive_eth_error_impl(input: DeriveInput) -> TokenStream {
    let attributes = match parse_calllike_attributes(&input, "etherror") {
        Ok(attributes) => attributes,
        Err(errors) => return errors,
    };

    let error_name = attributes.name.map(|(s, _)| s).unwrap_or_else(|| input.ident.to_string());

    let mut error = if let Some((src, span)) = attributes.abi {
        let raw_function_sig = src.trim_start_matches("error ").trim_start();
        // try to parse as solidity error
        if let Ok(fun) = HumanReadableParser::parse_error(&src) {
            fun
        } else {
            // try to determine the abi by using its fields at runtime
            return match derive_trait_impls_with_abi_type(
                &input,
                &error_name,
                Some(raw_function_sig),
            ) {
                Ok(derived) => derived,
                Err(err) => {
                    Error::new(span, format!("Unable to determine ABI for `{src}` : {err}"))
                        .to_compile_error()
                }
            }
        }
    } else {
        // try to determine the abi by using its fields at runtime
        return match derive_trait_impls_with_abi_type(&input, &error_name, None) {
            Ok(derived) => derived,
            Err(err) => err.to_compile_error(),
        }
    };
    error.name = error_name.clone();
    let abi = error.abi_signature();
    let selector = utils::selector(error.selector());
    let decode_impl = derive_decode_impl_from_params(&error.inputs, ident("EthError"));

    derive_trait_impls(&input, &error_name, quote! {#abi.into()}, Some(selector), decode_impl)
}

/// Use the `AbiType` trait to determine the correct `ParamType` and signature at runtime
fn derive_trait_impls_with_abi_type(
    input: &DeriveInput,
    function_call_name: &str,
    abi_signature: Option<&str>,
) -> Result<TokenStream, Error> {
    let abi_signature = if let Some(abi) = abi_signature {
        quote! {#abi}
    } else {
        utils::derive_abi_signature_with_abi_type(input, function_call_name, "EthError")?
    };

    let abi_signature = quote! {
         #abi_signature.into()
    };
    let decode_impl = derive_decode_impl_with_abi_type(input, ident("EthError"))?;
    Ok(derive_trait_impls(input, function_call_name, abi_signature, None, decode_impl))
}

/// Generates the EthError implementation
pub fn derive_trait_impls(
    input: &DeriveInput,
    function_call_name: &str,
    abi_signature: TokenStream,
    selector: Option<TokenStream>,
    decode_impl: TokenStream,
) -> TokenStream {
    // the ethers crates to use
    let core_crate = ethers_core_crate();
    let contract_crate = ethers_contract_crate();
    let struct_name = &input.ident;

    let selector = selector.unwrap_or_else(|| {
        quote! {
             #core_crate::utils::id(Self::abi_signature())
        }
    });

    let etherror_impl = quote! {
        impl #contract_crate::EthError for #struct_name {

            fn error_name() -> ::std::borrow::Cow<'static, str> {
                #function_call_name.into()
            }

            fn selector() -> #core_crate::types::Selector {
                #selector
            }

            fn abi_signature() -> ::std::borrow::Cow<'static, str> {
                #abi_signature
            }
        }

    };
    let codec_impl = derive_codec_impls(input, decode_impl, ident("EthError"));

    quote! {
        #etherror_impl
        #codec_impl
    }
}
