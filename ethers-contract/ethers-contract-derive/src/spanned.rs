//! Provides implementation for helpers used in parsing `TokenStream`s where the
//! data ultimately does not care about its `Span` information, but it is useful
//! during intermediate processing.

use proc_macro2::Span;
use std::ops::Deref;
use syn::parse::{Parse, ParseStream, Result as ParseResult};

/// Trait that abstracts functionality for inner data that can be parsed and
/// wrapped with a specific `Span`.
pub trait ParseInner: Sized {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)>;
}

impl<T: Parse> ParseInner for T {
    fn spanned_parse(input: ParseStream) -> ParseResult<(Span, Self)> {
        Ok((input.span(), T::parse(input)?))
    }
}

impl<T: ParseInner> Parse for Spanned<T> {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let (span, value) = T::spanned_parse(input)?;
        Ok(Spanned(span, value))
    }
}

/// A struct that captures `Span` information for inner parsable data.
#[cfg_attr(test, derive(Clone, Debug))]
pub struct Spanned<T>(Span, T);

impl<T> Spanned<T> {
    /// Retrieves the captured `Span` information for the parsed data.
    pub fn span(&self) -> Span {
        self.0
    }

    /// Retrieves the inner data.
    pub fn into_inner(self) -> T {
        self.1
    }
}

impl<T> Deref for Spanned<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.1
    }
}
