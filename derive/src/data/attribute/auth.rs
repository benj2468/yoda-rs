use std::convert::TryFrom;
use syn::{punctuated::Punctuated, token::Comma, ExprAssign};

#[derive(PartialEq, Eq, Default, Debug, Clone)]
pub struct AuthAttribute {
    pub mutate: Vec<auth::Role>,
    pub query: Vec<auth::Role>,
}

impl syn::parse::Parse for AuthAttribute {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        let punctuated: Punctuated<ExprAssign, Comma> =
            Punctuated::parse_separated_nonempty(input)?;

        let (mutate, query): (Vec<_>, Vec<_>) =
            punctuated
                .iter()
                .partition(|expr| match expr.left.as_ref() {
                    syn::Expr::Path(path) => path
                        .path
                        .get_ident()
                        .map(|ident| *ident == "mutate")
                        .unwrap_or_default(),
                    _ => unimplemented!("Only path expressions"),
                });

        let (mutate, query) = (from_expr(mutate.first()), from_expr(query.first()));

        Ok(Self { mutate, query })
    }
}

fn from_expr(expr: Option<&&ExprAssign>) -> Vec<auth::Role> {
    expr.map(|expr| match expr.right.as_ref() {
        syn::Expr::Array(array) => array
            .elems
            .iter()
            .map(|element| match element {
                syn::Expr::Lit(lit) => match &lit.lit {
                    syn::Lit::Str(s) => {
                        auth::Role::try_from(&s.value()).unwrap_or_else(|e| panic!("{}", e))
                    }
                    _ => unimplemented!("Elements of array must be Literals str"),
                },
                _ => unimplemented!("Elements of array must be Literals"),
            })
            .collect(),
        _ => unimplemented!("Expr must be an array"),
    })
    .unwrap_or_default()
}
