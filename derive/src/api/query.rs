use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::*;
use crate::DeriveData;
use heck::SnakeCase;

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let ident = Ident::new(format!("{}Query", base.to_string()).as_str(), base.span());

    let find = derive_find(input);

    let search = derive_search(input);

    quote! {
        #[derive(Default)]
        pub struct #ident;

        #[Object]
        impl #ident {
            #find

            #search
        }
    }
}

fn derive_find(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;
    let func_name = Ident::new(
        format!("find_{}", base.to_string().to_snake_case()).as_str(),
        base.span(),
    );

    let store = Ident::new(format!("{}Store", base.to_string()).as_str(), base.span());

    let query_permitted = input.auth_attribute().query;

    quote! {
        async fn #func_name(&self, ctx: &Context<'_>, id: String) -> Result<#base> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            identity.is_authorized(vec![#(#query_permitted),*])?;

            let dels = store::sql::Driver::query::<#base, #store>(pool, &id)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            Ok(dels.into_iter().rollup())
        }
    }
}

fn derive_search(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;
    let func_name = Ident::new(
        format!("search_{}", base.to_string().to_snake_case()).as_str(),
        base.span(),
    );

    let search = Ident::new(format!("{}Search", base.to_string()).as_str(), base.span());

    let query_permitted = input.auth_attribute().query;

    quote! {
        /// Search for #base
        /// ### Defaults
        /// Cursor: 0
        ///
        /// Limit: 100
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            name: Option<String>,
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<Connection<usize, #base>> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            identity.is_authorized(vec![#(#query_permitted),*])?;

            let doc = #search { name };

            let pagination = PaginationOption {
                skip: cursor.unwrap_or("0".into()).parse()?,
                limit: limit.unwrap_or(100),
            };

            let edges: Vec<Edge<usize, #base, EmptyFields>> =
                store::sql::Driver::search(pool, doc, pagination)
                    .await?
                    .into_iter()
                    .map(|org| org.into())
                    .enumerate()
                    .map(|(i, org)| Edge::new(i + pagination.skip, org))
                    .collect();

            let end_cursor = pagination.skip + edges.len();

            let PaginationOption { skip, limit } = pagination;

            let mut connection = Connection::new(skip > 0, true);

            connection.append(edges);

            Ok(connection)
        }
    }
}
