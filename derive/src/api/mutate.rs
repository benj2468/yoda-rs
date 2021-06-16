use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::*;
use crate::DeriveData;
use heck::{CamelCase, SnakeCase};

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let ident = Ident::new(format!("{}Mutate", base.to_string()).as_str(), base.span());

    let new = derive_new(input);

    let update = derive_update(input);

    quote! {
        #[derive(Default)]
        pub struct #ident;

        #[Object]
        impl #ident {
            #new

            #update
        }
    }
}

fn derive_new(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;
    let func_name = Ident::new(
        format!("new_{}", base.to_string().to_snake_case()).as_str(),
        base.span(),
    );

    let store = Ident::new(format!("{}Store", base.to_string()).as_str(), base.span());

    let params = input.fields.iter().map(|field| {
        let Field {
            ident,
            attributes,
            ty,
            ..
        } = field;
        let ty = if attributes.contains(&Attribute::Struct) {
            let ident = Ident::new(
                format!("{}Input", ty.ty_str()).to_camel_case().as_str(),
                ident.span(),
            );
            match ty.wrapper {
                Wrapper::Option => quote! { Option<#ident> },
                Wrapper::Vec => quote! { Option<Vec<#ident>> },
            }
        } else {
            field.wrapped()
        };
        quote! {
            #ident: #ty
        }
    });

    let fields = input
        .fields
        .iter()
        .filter(|field| !field.is_identifier())
        .map(|field| {
            let name = &field.ident;

            if field.attributes.contains(&Attribute::Struct) {
                match field.ty.wrapper {
                    Wrapper::Vec => quote! {
                        #name: #name.unwrap_or_default().into_iter().map(|val| val.into()).collect()
                    },
                    Wrapper::Option => {
                        quote! {
                            #name: #name.map(|val| val.into())
                        }
                    }
                }
            } else {
                quote! { #name }
            }
        });

    let mutate_permitted = input.auth_attribute().mutate;

    quote! {
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            #(#params,)*
        ) -> Result<atoms::Identifier> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            identity.is_authorized(vec![#(#mutate_permitted),*])?;

            let mut identifier: Vec<_> = identifier
                .unwrap_or_default()
                .into_iter()
                .map(|id| id.into())
                .collect();

            let new_identifier = atoms::Identifier::new(
                atoms::IdentifierSystem::Yoda,
                atoms::IdentifierTier::Primary,
            );
            identifier.push(new_identifier.clone());

            let doc = #base {
                identifier,
                #(#fields,)*
            };

            let mut transaction = pool.begin().await?;

            store::sql::Driver::delta::<#store>(
                &mut transaction,
                &new_identifier.value,
                doc.clone().into(),
                identity,
            )
            .await?;

            store::sql::Driver::project::<#store>(
                &mut transaction,
                &new_identifier.value,
                doc.into(),
            )
            .await?;

            transaction.commit().await?;

            Ok(new_identifier)
        }
    }
}

fn derive_update(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;
    let func_name = Ident::new(
        format!("update_{}", base.to_string().to_snake_case()).as_str(),
        base.span(),
    );

    let store = Ident::new(format!("{}Store", base.to_string()).as_str(), base.span());

    let params = input.fields.iter().map(|field| {
        let Field { ident, .. } = field;
        let delta = Ident::new(
            format!(
                "Delta{}{}",
                base.to_string().to_camel_case(),
                ident.to_string().to_camel_case()
            )
            .as_str(),
            ident.span(),
        );
        quote! {
            #ident: Option<#delta>
        }
    });

    let fields = input.fields.iter().map(|field| {
        let name = &field.ident;

        quote! { #name: #name.map(|del| del.into()) }
    });

    let mutate_permitted = input.auth_attribute().mutate;

    quote! {
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            id: async_graphql::ID,
            #(#params,)*
        ) -> Result<#base> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            identity.is_authorized(vec![#(#mutate_permitted),*])?;

            let delta = #store {
                #(#fields,)*
            };

            let mut transaction = pool.begin().await?;

            store::sql::Driver::delta(&mut transaction, id.as_str(), delta.clone(), identity).await?;

            let mut current_doc: #base =
                store::sql::Driver::query_proj(&mut transaction, id.as_str()).await?;

            current_doc.apply(delta);

            store::sql::Driver::project::<#store>(
                &mut transaction,
                id.as_str(),
                current_doc.clone().into(),
            )
            .await?;

            transaction.commit().await?;

            Ok(current_doc)
        }
    }
}
