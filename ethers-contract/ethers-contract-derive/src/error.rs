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
pub(crate) fn derive_eth_error_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    let attributes = parse_calllike_attributes!(input, "etherror");

    let error_name = attributes.name(&input.ident);
    let mut error = if let Some((src, span)) = attributes.abi() {
        let raw_function_sig = src.trim_start_matches("error ").trim_start();
        // try to parse as solidity error
        match HumanReadableParser::parse_error(&src) {
            Ok(solidity_error) => solidity_error,
            Err(parse_err) => {
                return match derive_trait_impls_with_abi_type(
                    &input,
                    &error_name,
                    Some(raw_function_sig),
                ) {
                    Ok(derived) => Ok(derived),
                    Err(err) => {
                        Err(Error::new(span, format!("Unable to determine ABI for `{src}`: {err}")))
                    }
                    .map_err(|e| {
                        let mut error = Error::new(span, parse_err);
                        error.combine(Error::new(span, e));
                        error
                    }),
                }
            }
        }
    } else {
        // try to determine the abi by using its fields at runtime
        return derive_trait_impls_with_abi_type(&input, &error_name, None)
    };
    error.name = error_name.clone();

    let sig = error.abi_signature();
    let selector = utils::selector(error.selector());
    let decode_impl = derive_decode_impl_from_params(&error.inputs, ident("EthError"));

    derive_trait_impls(&input, &error_name, quote!(#sig.into()), Some(selector), decode_impl)
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
        utils::abi_signature_with_abi_type(input, function_call_name, "EthError")?
    };
    abi_signature.extend(quote!(.into()));
    let decode_impl = derive_decode_impl_with_abi_type(input, ident("EthError"))?;
    derive_trait_impls(input, function_call_name, abi_signature, None, decode_impl)
}

/// Generates the EthError implementation
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

    let etherror_impl = quote! {
        impl #ethers_contract::EthError for #struct_name {
            fn error_name() -> ::std::borrow::Cow<'static, str> {
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
    let codec_impl = derive_codec_impls(input, decode_impl, ident("EthError"))?;

    Ok(quote! {
        #etherror_impl
        #codec_impl
    })
}
