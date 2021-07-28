use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, GenericArgument, Ident, PathArguments, PathSegment, Visibility};

mod api;
mod support;

mod data;
pub(crate) use data::*;

#[proc_macro_derive(Api, attributes(construct, auth, searchable))]
pub fn derive_api(input: TokenStream) -> TokenStream {
    api::derive(input.into())
}

#[proc_macro_attribute]
#[allow(non_snake_case)]
pub fn Support(_attr: TokenStream, input: TokenStream) -> TokenStream {
    support::derive(input.into())
}

#[cfg(test)]
mod test;
