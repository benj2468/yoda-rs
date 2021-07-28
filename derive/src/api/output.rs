use heck::CamelCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::*;
use crate::DeriveData;

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let field_hash_structs = derive_field_hash_structs(&input);

    let resolvers = derive_resolvers(&input);

    quote! {
        #(#field_hash_structs)*

        #[async_graphql::Object]
        impl #base {
            #(#resolvers)*
        }
    }
}

fn with_hash_ident(base: &Ident, field: &Field) -> Ident {
    Ident::new(
        format!(
            "{}{}WithHash",
            base.to_string().to_camel_case(),
            field.ident.to_string().to_camel_case()
        )
        .as_str(),
        field.ident.span(),
    )
}

fn derive_field_hash_structs(input: &DeriveData) -> Vec<TokenStream2> {
    let base = &input.ident;

    input
        .fields
        .iter()
        .map(|field| {
            let ident = with_hash_ident(base, field);

            let ty = field.wrapped();

            quote! {
                #[derive(async_graphql::SimpleObject)]
                struct #ident {
                    value: #ty,
                    hash: String,
                }
            }
        })
        .collect()
}

fn derive_resolvers(input: &DeriveData) -> Vec<TokenStream2> {
    let base = &input.ident;
    input
        .fields
        .iter()
        .map(|field| {
            let Field { ident, ty, .. } = field;

            let output_ty = with_hash_ident(base, field);

            let hashing = match ty.wrapper {
                Wrapper::Vec | Wrapper::None => quote! {
                    value.hash(&mut hasher);
                },
                Wrapper::Option => {
                    quote! {
                        if let Some(val) = value {
                            val.hash(&mut hasher)
                        }
                    }
                }
            };
            quote! {
                async fn #ident(&self, ctx: &Context<'_>,) -> Result<#output_ty> {
                    use std::hash::{Hasher, Hash};

                    let value = &self.#ident;

                    let mut hasher = std::collections::hash_map::DefaultHasher::new();
                    #hashing
                    let hash = hasher.finish().to_string();

                    Ok(#output_ty {
                        value: value.clone(),
                        hash
                    })
                }
            }
        })
        .collect()
}
