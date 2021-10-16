//! Helper functions for deriving `EthCall`

use ethers_contract_abigen::{ethers_contract_crate, ethers_core_crate};
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned as _;
use syn::DeriveInput;

/// Generates the `EthEvent` trait support
pub(crate) fn derive_eth_call_impl(input: DeriveInput) -> TokenStream {
    quote! {}
}
