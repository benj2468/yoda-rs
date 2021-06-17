#![allow(unused_variables)]
use async_graphql::MergedObject;
use async_graphql::{
    connection::{Connection, Edge, EmptyFields},
    Context, Object, Result,
};
use atoms::delta::Rollup;
use atoms::delta::{check_delta, Del, Delta, Store};
use atoms::pagination::PaginationOption;
use atoms::search::{Search, SqlQuery};
use atoms::IdentifierInput;
use atoms::Tag;
use derive::Api;

#[derive(Default, Clone, Debug, Api)]
#[auth(
    mutate = ["admin", "service"],
    query = ["admin", "service", "user"]
)]
struct Account {
    #[construct]
    identifier: Vec<atoms::Identifier>,
    #[searchable]
    email: Option<String>,
    password: Option<String>,
    first_name: Option<String>,
    last_name: Option<String>,
    interests: Vec<Tag>,
}

#[derive(Default, MergedObject)]
pub struct AcctQuery(AccountQuery);

#[derive(Default, MergedObject)]
pub struct AcctMutate(AccountMutate);
