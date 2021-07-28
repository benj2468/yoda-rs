use super::*;
use async_graphql::{InputObject, SimpleObject};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Api)]
#[auth(
    mutate = ["admin", "service", "self"],
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
    #[construct]
    transactions: Vec<Reference>,
    #[construct]
    payment_method: Vec<Reference>,
    #[construct]
    address: Vec<Address>,
}

#[Support]
pub struct Address {
    number: Option<u32>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
    postal_code: Option<String>,
}

#[derive(Default, MergedObject)]
pub struct AcctQuery(AccountQuery);

#[derive(Default, MergedObject)]
pub struct AcctMutate(AccountMutate);
