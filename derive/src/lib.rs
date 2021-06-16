use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Data, DeriveInput, GenericArgument, Ident, PathArguments, PathSegment, Visibility};

mod api;
mod atom;

mod data;
pub(crate) use data::*;

#[proc_macro_derive(Api, attributes(construct))]
pub fn derive_api(input: TokenStream) -> TokenStream {
    api::derive(input.into())
}

#[cfg(test)]
mod test;
