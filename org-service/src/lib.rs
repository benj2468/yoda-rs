use async_graphql::MergedObject;
use atoms::delta::{check_delta, Del, Delta, Store};
use atoms::search::{Search, SqlQuery};
use atoms::Tag;
use chrono::prelude::{DateTime, Utc};
use derive::Api;

use atoms::pagination::PaginationOption;

use async_graphql::{
    connection::{Connection, Edge, EmptyFields},
    Context, Object, Result,
};
use atoms::delta::Rollup;
use atoms::IdentifierInput;

#[derive(Default, Clone, Debug, Api)]
struct Organization {
    #[construct]
    identifier: Vec<atoms::Identifier>,
    name: Option<String>,
    mission: Option<String>,
    description: Option<String>,
    established: Option<DateTime<Utc>>,
    tag: Vec<Tag>,
    ceo: Option<String>,
}

pub struct OrganizationSearch {
    name: Option<String>,
}

impl Search<OrganizationStore> for OrganizationSearch {
    fn ty() -> String {
        "Organization".into()
    }
    fn array_splits(&self) -> Vec<String> {
        Default::default()
    }
    fn paths(&self) -> Vec<String> {
        let mut res = vec![];

        if self.name.is_some() {
            res.push("body -> 'name' ->> 'end' LIKE $".into())
        }

        res
    }
    fn bind_args<'a>(&self, query: SqlQuery<'a>) -> SqlQuery<'a> {
        let query = if let Some(name) = self.name.clone() {
            query.bind(format!("%{}%", name))
        } else {
            query
        };

        query
    }
}

#[derive(Default, MergedObject)]
pub struct OrgQuery(OrganizationQuery);

#[derive(Default, MergedObject)]
pub struct OrgMutate(OrganizationMutate);