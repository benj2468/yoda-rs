#![allow(unused_variables)]
use async_graphql::{
    connection::{Connection, Edge, EmptyFields},
    Context, MergedObject, Object, Result,
};
use atoms::{
    delta::{check_delta, Del, Delta, Rollup, Store},
    pagination::PaginationOption,
    search::{Search, SqlQuery},
    *,
};
use derive::{Api, Support};

mod account;
mod organization;
mod transaction;

#[derive(Default, MergedObject)]
pub struct Query(
    account::AcctQuery,
    organization::OrgQuery,
    transaction::TxnQuery,
);

#[derive(Default, MergedObject)]
pub struct Mutate(
    account::AcctMutate,
    organization::OrgMutate,
    transaction::TxnMutate,
);
