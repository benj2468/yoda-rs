use heck::CamelCase;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;

use super::*;
use crate::DeriveData;

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let field_deltas = derive_field_deltas(&input);

    quote! {
        #(#field_deltas)*
    }
}

fn derive_field_deltas(input: &DeriveData) -> Vec<TokenStream2> {
    let base = &input.ident;

    input
        .fields
        .iter()
        .map(|field| {
            let Field {
                ident,
                ty,
                attributes,
                ..
            } = field;

            if !attributes.contains(&Attribute::Struct) {
                let ident = Ident::new(
                    format!(
                        "Delta{}{}",
                        base.to_string().to_camel_case(),
                        ident.to_string().to_camel_case()
                    )
                    .as_str(),
                    ident.span(),
                );
                let ty = match ty.wrapper {
                    Wrapper::Option => {
                        let ty = &ty.ty;
                        quote! { #ty }
                    }
                    Wrapper::Vec | Wrapper::None => field.wrapped(),
                };

                quote! {
                    #[derive(async_graphql::InputObject)]
                    pub struct #ident {
                        start: Option<String>,
                        end: Option<#ty>,
                    }

                    impl From<#ident> for Delta<#ty> {
                        fn from(delta: #ident) -> Self {
                            Self {
                                start: delta.start,
                                end: delta.end,
                            }
                        }
                    }
                }
            } else {
                let ident = Ident::new(
                    format!(
                        "Delta{}{}",
                        base.to_string().to_camel_case(),
                        ident.to_string().to_camel_case()
                    )
                    .as_str(),
                    ident.span(),
                );

                let conversion = match &ty.wrapper {
                    Wrapper::Vec => quote! { delta.end.map(|vec| vec.into_iter().map(|id| id.into()).collect()) },
                    Wrapper::Option | Wrapper::None => quote! {
                        delta.end.map(|e| e.into())
                    },
                };

                let input_ty = {
                    let ident = Ident::new(
                        format!("{}Input", ty.ty_str()).to_camel_case().as_str(),
                        ty.ty.span(),
                    );

                    match ty.wrapper {
                        Wrapper::Vec => quote! { Vec<#ident> },
                        _ => unimplemented!(),
                    }
                };

                let store_ty = match ty.wrapper {
                    Wrapper::Vec | Wrapper::None => field.wrapped(),
                    Wrapper::Option => {
                        let ty = &ty.ty;
                        quote! { #ty }
                    }
                };

                quote! {
                    #[derive(async_graphql::InputObject)]
                    pub struct #ident {
                        start: Option<String>,
                        end: Option<#input_ty>,
                    }

                    impl From<#ident> for Delta<#store_ty> {
                        fn from(delta: #ident) -> Self {
                            Self {
                                start: delta.start,
                                end: #conversion,
                            }
                        }
                    }
                }
            }
        })
        .collect()
}
