use super::*;
use heck::{CamelCase, SnakeCase};
use quote::quote;
use syn::{parse_quote, token::Token};

pub(crate) fn derive(input: DeriveData) -> TokenStream {
    let input_derived = derive_input(&input);

    let standard = derive_standard(&input);

    TokenStream::from(quote! {
        #standard

        #input_derived
    })
}

fn derive_standard(input: &DeriveData) -> TokenStream2 {
    let DeriveData { ident, fields, .. } = input;

    let fields = fields.iter().map(|field| {
        let ident = &field.ident;
        let wrapped = field.wrapped();

        quote! {
            #ident: #wrapped
        }
    });

    quote! {
        #[derive(
            Clone, Debug, Deserialize, Serialize, Hash, PartialEq, Eq, SimpleObject,
        )]
        pub struct #ident {
            #(#fields,)*
        }
    }
}

fn derive_input(input: &DeriveData) -> TokenStream2 {
    let DeriveData { ident, fields, .. } = input;

    let input_ident = Ident::new(&format!("{}Input", ident.to_string()), ident.span());

    let input_fields = fields.iter().map(|field| {
        let Field {
            ident,
            attributes,
            ty,
            ..
        } = field;
        let ty = if attributes.contains(&Attribute::Struct) {
            let ident = Ident::new(
                &format!("{}Input", ty.ty_str().to_camel_case()),
                ident.span(),
            );
            field.wrap_other(ident)
        } else {
            field.wrapped()
        };

        quote! {
            #ident: #ty
        }
    });

    let froms = fields.iter().map(|field| {
        let Field { ident, ty, .. } = field;
        let conversion = match ty.wrapper {
            Wrapper::Option => {
                quote! { input.#ident.map(|val| val.into()) }
            }
            Wrapper::Vec => {
                quote! { input.#ident.into_iter().map(|val| val.into()).collect() }
            }
            Wrapper::None => {
                quote! { input.#ident.into() }
            }
        };

        quote! {
            #ident: #conversion
        }
    });

    quote! {
        #[derive(Clone, Debug, InputObject)]
        pub struct #input_ident {
            #(#input_fields,)*
        }

        impl From<#input_ident> for #ident {
            fn from(input: #input_ident) -> Self {
                Self {
                    #(#froms,)*
                }
            }
        }
    }
}
