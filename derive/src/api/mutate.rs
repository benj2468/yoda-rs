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

    let params = input
        .fields
        .iter()
        .filter(|field| !field.is_identifier())
        .map(|field| {
            let Field { ident, .. } = field;
            let ty = field.wrapped();
            quote! {
                #ident: #ty
            }
        });

    let fields = input.fields.iter().map(|field| {
        let name = &field.ident;

        quote! { #name }
    });

    quote! {
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            identifier: Option<Vec<atoms::IdentifierInput>>,
            #(#params,)*
        ) -> Result<atoms::Identifier> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            // identity.is_authorized(auth::Action::CreateOrganization)?;

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
        let Field { ident, ty, .. } = field;
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

    quote! {
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            id: async_graphql::ID,
            #(#params,)*
        ) -> Result<#base> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            // identity.is_authorized(auth::Action::UpdateOrganization)?;

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