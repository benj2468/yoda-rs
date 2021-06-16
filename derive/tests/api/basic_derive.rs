use async_graphql::{
    connection::{Connection, Edge, EmptyFields},
    Context, Enum, Object, Result, SimpleObject,
};
use atoms::delta::Rollup;
use atoms::delta::*;
use atoms::pagination::PaginationOption;
use atoms::search::{Search, SqlQuery};

#[derive(SimpleObject, Default, Clone, Debug, derive::Api)]
pub struct Org {
    identifier: Vec<atoms::Identifier>,
    name: Option<String>,
    tag: Option<Vec<Tag>>,
}

#[derive(
    Enum, Debug, serde::Serialize, serde::Deserialize, std::hash::Hash, Clone, Copy, PartialEq, Eq,
)]
enum Tag {
    Value,
}

pub struct OrgSearch {
    name: Option<String>,
}

impl Search<OrgStore> for OrgSearch {
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

pub fn main() {}
