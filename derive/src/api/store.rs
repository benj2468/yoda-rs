use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::*;
use crate::DeriveData;

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let store_struct = derive_struct(&input);

    let impl_store = derive_store_trait(&input);

    let impl_del = derive_del_trait(&input);

    let translations = derive_translations(&input);

    quote! {
        #store_struct

        #translations

        #impl_store

        #impl_del
    }
}

fn derive_struct(input: &DeriveData) -> TokenStream2 {
    let DeriveData { ident, fields, .. } = input;

    let ident = Ident::new(format!("{}Store", ident.to_string()).as_str(), ident.span());

    let fields = fields.iter().map(|field| {
        let Field { ident, ty, .. } = field;

        let Type { ty, wrapper, .. } = ty;

        match wrapper {
            Wrapper::Option => quote! { #ident: Option<Delta<#ty>> },
            Wrapper::Vec => quote! { #ident: Option<Delta<Vec<#ty>>> },
        }
    });

    quote! {
        #[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
        pub struct #ident {
            #(#fields,)*
        }
    }
}

fn derive_store_trait(input: &DeriveData) -> TokenStream2 {
    let DeriveData { ident, fields, .. } = input;

    let ident_str = ident.to_string();

    let ident = Ident::new(format!("{}Store", ident_str).as_str(), ident.span());

    let identifier = match fields
        .iter()
        .find(|field| field.is_identifier())
        .expect("Every struct must have an identifier field")
        .ty
        .wrapper
    {
        Wrapper::Vec => {
            quote! {
                self.identifier
                    .as_ref()
                    .and_then(|delta| delta.end.as_ref())
                    .and_then(|vec| vec.into_iter().find(|id| id.is_primary()))
                    .map(|id| id.to_string())
                    .unwrap_or_default()
            }
        }
        _ => unimplemented!("Identifier must be an array"),
    };

    quote! {
        impl Store for #ident {
            fn ty() -> String {
                #ident_str.into()
            }
            fn identifier(&self) -> String {
                #identifier
            }
        }
    }
}
fn derive_del_trait(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let ident_str = base.to_string();
    let ident = Ident::new(format!("{}Store", ident_str).as_str(), base.span());

    let fields = input.fields.iter().map(|field| {
        let ident = &field.ident;

        quote! { #ident }
    });

    let apply_deltas = input.fields.iter().map(|field| {
        let ident = &field.ident;
        if field.is_identifier() {
            return quote! {};
        };
        match field.ty.wrapper {
            Wrapper::Vec => quote! {
                if let Some(new) = check_delta(Some(&#ident), del.#ident.unwrap_or_default()) {
                    *#ident = new
                }
            },
            Wrapper::Option => quote! {
                if let Some(new) = check_delta(#ident.as_ref(), del.#ident.unwrap_or_default()) {
                    #ident.replace(new);
                }
            },
        }
    });

    quote! {
        impl Del<#ident> for #base {
            fn ty() -> String {
                #ident_str.into()
            }

            fn apply(&mut self, del: #ident) {
                let Self { #(#fields,)* } = self;

                #(#apply_deltas)*
            }
        }
    }
}

fn derive_translations(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let ident_str = base.to_string();
    let ident = Ident::new(format!("{}Store", ident_str).as_str(), base.span());

    let from_base = {
        let fields = input.fields.iter().map(|field| {
            let name = &field.ident;
            match field.ty.wrapper {
                Wrapper::Option => quote! { #name: Delta::init(org.#name) },
                Wrapper::Vec => quote! { #name: Delta::init(Some(org.#name)) },
            }
        });

        quote! {
            impl From<#base> for #ident {
                fn from(org: #base) -> Self {
                    Self {
                        #(#fields,)*
                    }
                }
            }
        }
    };

    let from_store = {
        let fields = input.fields.iter().map(|field| {
            let name = &field.ident;
            match field.ty.wrapper {
                Wrapper::Option => {
                    quote! { #name: store.#name.and_then(|del| del.end.into()) }
                }
                Wrapper::Vec => {
                    quote! { #name: store.#name.and_then(|del| del.end).unwrap_or_default() }
                }
            }
        });

        quote! {
            impl From<#ident> for #base {
                fn from(store: #ident) -> Self {
                    Self {
                        #(#fields,)*
                    }
                }
            }
        }
    };

    quote! {

        #from_base

        #from_store
    }
}
