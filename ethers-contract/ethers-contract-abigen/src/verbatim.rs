use proc_macro2::{Punct, TokenStream};
use quote::{quote, TokenStreamExt};
use std::collections::BTreeMap;
use syn::Path;

/// Generates the constructor tokens of the given type.
pub(crate) fn generate<T: Verbatim>(t: &T, ethers_core: &Path) -> TokenStream {
    let mut s = TokenStream::new();
    t.to_tokens(&mut s, ethers_core);
    s
}

/// Generates the constructor tokens.
pub(crate) trait Verbatim {
    fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path);
}

struct ToTokensCompat<'a, T: Verbatim>(&'a T, &'a Path);

impl<T: Verbatim> quote::ToTokens for ToTokensCompat<'_, T> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens, self.1)
    }
}

impl Verbatim for String {
    fn to_tokens(&self, s: &mut TokenStream, _: &Path) {
        if self.is_empty() {
            s.extend(quote!(::std::string::String::new()))
        } else {
            s.extend(quote!(::std::borrow::ToOwned::to_owned(#self)))
        }
    }
}

impl Verbatim for bool {
    #[inline]
    fn to_tokens(&self, s: &mut TokenStream, _: &Path) {
        quote::ToTokens::to_tokens(self, s)
    }
}

impl Verbatim for usize {
    #[inline]
    fn to_tokens(&self, s: &mut TokenStream, _: &Path) {
        quote::ToTokens::to_tokens(self, s)
    }
}

impl<T: ?Sized + Verbatim> Verbatim for Box<T> {
    fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
        let mut t = TokenStream::new();
        (**self).to_tokens(&mut t, ethers_core);
        s.extend(quote!(::std::boxed::Box::new(#t)));
    }
}

impl<T: Verbatim> Verbatim for Vec<T> {
    fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
        let iter = self.iter().map(|x| ToTokensCompat(x, ethers_core));
        s.extend(quote!(::std::vec![#(#iter),*]));
    }
}

impl<K: Verbatim, V: Verbatim> Verbatim for BTreeMap<K, V> {
    fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
        if self.is_empty() {
            return s.extend(quote!(::std::collections::BTreeMap::new()))
        }

        let iter = self.iter().map(|(k, v)| {
            let mut s = TokenStream::new();
            k.to_tokens(&mut s, ethers_core);
            s.append(Punct::new(',', proc_macro2::Spacing::Alone));
            v.to_tokens(&mut s, ethers_core);
            quote! { (#s), }
        });
        s.extend(quote!(::core::convert::From::from([#(#iter)*])));
    }
}

impl<T: Verbatim> Verbatim for Option<T> {
    fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
        let tts = match self {
            Some(t) => {
                let mut s = TokenStream::new();
                t.to_tokens(&mut s, ethers_core);
                quote!(::core::option::Option::Some(#s))
            }
            None => quote!(::core::option::Option::None),
        };
        s.extend(tts);
    }
}

macro_rules! derive_verbatim {
    () => {};

    (struct $name:ident { $($field:ident),* $(,)? } $($rest:tt)*) => {
        impl Verbatim for ethers_core::abi::ethabi::$name {
            #[allow(deprecated)]
            fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
                $(
                    let $field = ToTokensCompat(&self.$field, ethers_core);
                )*
                s.extend(quote! {
                    #ethers_core::abi::ethabi::$name {
                        $($field: #$field,)*
                    }
                });
            }
        }
        derive_verbatim!($($rest)*);
    };

    (enum $name:ident { $($variant:ident $(($($field:ident),* $(,)?))?),* $(,)? } $($rest:tt)*) => {
        impl Verbatim for ethers_core::abi::ethabi::$name {
            #[allow(deprecated)]
            fn to_tokens(&self, s: &mut TokenStream, ethers_core: &Path) {
                match self {$(
                    Self::$variant $(($($field),*))? => {
                        $($(
                            let $field = ToTokensCompat($field, ethers_core);
                        )*)?
                        s.extend(quote! {
                            #ethers_core::abi::ethabi::$name::$variant $(($(#$field),*))?
                        });
                    }
                )*}
            }
        }
        derive_verbatim!($($rest)*);
    };
}

derive_verbatim! {
    struct Contract { constructor, functions, events, errors, receive, fallback, }
    struct Constructor { inputs }
    struct Function { name, inputs, outputs, constant, state_mutability }
    struct Event { name, inputs, anonymous }
    struct AbiError { name, inputs }
    struct Param { name, kind, internal_type }
    struct EventParam { name, kind, indexed }

    enum ParamType {
        Address,
        Bytes,
        Int(size),
        Uint(size),
        Bool,
        String,
        Array(inner),
        FixedBytes(size),
        FixedArray(inner, size),
        Tuple(inner),
    }
    enum StateMutability {
        Pure,
        View,
        NonPayable,
        Payable,
    }
}
