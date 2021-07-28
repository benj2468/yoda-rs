use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use super::*;
use crate::DeriveData;
use heck::SnakeCase;

pub(crate) fn derive(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let ident = Ident::new(format!("{}Query", base.to_string()).as_str(), base.span());

    let find = derive_find(input);

    let (search_struct, search) = if input
        .fields
        .iter()
        .any(|field| field.attributes.contains(&Attribute::Search))
    {
        let search_struct = derive_search_struct(input);

        let search = derive_search_func(input);

        (search_struct, search)
    } else {
        Default::default()
    };

    quote! {

        #search_struct

        #[derive(Default)]
        pub struct #ident;

        #[Object]
        impl #ident {
            #find

            #search
        }
    }
}

fn search_struct_ident(input: &DeriveData) -> Ident {
    let base = &input.ident;
    Ident::new(format!("{}Search", base.to_string()).as_str(), base.span())
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
            identity.is_authorized(auth::Action::Query, vec![#(#query_permitted),*])?;

            let dels = store::sql::Driver::query::<#base, #store>(pool, &id)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?;

            Ok(dels.into_iter().rollup())
        }
    }
}

fn derive_search_func(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;
    let func_name = Ident::new(
        format!("search_{}", base.to_string().to_snake_case()).as_str(),
        base.span(),
    );

    let search = search_struct_ident(input);

    let query_permitted = input.auth_attribute().query;

    let search_params = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&Attribute::Search))
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let ty = &ty.ty;
            quote! {
                #ident: Option<#ty>
            }
        });

    let search_idents = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&Attribute::Search))
        .map(|field| &field.ident);

    let search_for_comment = format!("Search for {}", base.to_string());

    quote! {
        #[doc = #search_for_comment]
        /// ### Defaults
        /// Cursor: 0
        ///
        /// Limit: 100
        #[allow(clippy::too_many_arguments)]
        async fn #func_name(
            &self,
            ctx: &Context<'_>,
            #(#search_params,)*
            cursor: Option<String>,
            limit: Option<usize>,
        ) -> Result<Connection<usize, #base>> {
            let identity = ctx.data::<auth::Identity>()?;
            let pool = ctx.data::<sqlx::PgPool>()?;
            identity.is_authorized(auth::Action::All, vec![#(#query_permitted),*])?;

            let doc = #search { #(#search_idents,)* };

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

fn derive_search_struct(input: &DeriveData) -> TokenStream2 {
    let base = &input.ident;

    let search = search_struct_ident(input);
    let store = Ident::new(format!("{}Store", base.to_string()).as_str(), base.span());

    let base_str = base.to_string();

    let search_fields = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&&Attribute::Search))
        .map(|field| {
            let Field { ident, ty, .. } = field;
            let ty = &ty.ty;
            quote! {
                #ident: Option<#ty>
            }
        });

    let splits = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&&Attribute::Search))
        .filter_map(|field| {
            let Field { ty, .. } = field;

            match ty.wrapper {
                Wrapper::Option | Wrapper::None => None,
                Wrapper::Vec => Some(""),
            }
        });

    let paths = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&&Attribute::Search))
        .map(|field| {
            let Field { ident, ty, .. } = field;

            let path = match ty.wrapper {
                Wrapper::Option | Wrapper::None => {
                    format!("body -> '{}' ->> 'end' LIKE $", ident.to_string())
                }
                Wrapper::Vec => format!("{}_array ->> 'end' LIKE $", ident.to_string()),
            };

            quote! {
                if self.#ident.is_some() {
                    res.push(#path.into())
                }
            }
        });

    let bindings = input
        .fields
        .iter()
        .filter(|field| field.attributes.contains(&&Attribute::Search))
        .map(|field| {
            let Field { ident, .. } = field;

            quote! {
                let query = if let Some(#ident) = self.#ident.as_ref() {
                    query.bind(format!("%{}%", #ident))
                } else { query };
            }
        });

    quote! {
        pub struct #search {
            #(#search_fields,)*
        }

        impl Search<#store> for #search {
            fn ty() -> String {
                #base_str.into()
            }
            fn array_splits(&self) -> Vec<String> {
                vec![#(#splits,)*]
            }
            fn paths(&self) -> Vec<String> {
                let mut res = vec![];

                #(#paths)*

                res
            }
            fn bind_args<'a>(&self, query: SqlQuery<'a>) -> SqlQuery<'a> {

                #(#bindings)*

                query
            }
        }
    }
}
