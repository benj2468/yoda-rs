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
use atoms::Tag;
use atoms::{IdentifierInput, ReferenceInput};
use chrono::prelude::{DateTime, Utc};
use derive::Api;

#[derive(Default, Clone, Debug, Api)]
#[auth(
    mutate = ["admin", "organization"],
    query = ["admin", "organization", "user"]
)]
struct Organization {
    #[construct]
    identifier: Vec<atoms::Identifier>,
    #[searchable]
    name: Option<String>,
    mission: Option<String>,
    description: Option<String>,
    established: Option<DateTime<Utc>>,
    tag: Vec<Tag>,
    ceo: Option<String>,
    #[construct]
    managing_entity: Vec<atoms::Reference>,
}

#[derive(Default, MergedObject)]
pub struct OrgQuery(OrganizationQuery);

#[derive(Default, MergedObject)]
pub struct OrgMutate(OrganizationMutate);
