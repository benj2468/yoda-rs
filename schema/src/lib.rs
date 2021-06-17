use account_service::{AcctMutate, AcctQuery};
use async_graphql::{EmptySubscription, MergedObject};
use org_service::{OrgMutate, OrgQuery};

#[derive(Default, MergedObject)]
pub struct Query(OrgQuery, AcctQuery);

#[derive(Default, MergedObject)]
pub struct Mutate(OrgMutate, AcctMutate);

pub type YodaSchema = async_graphql::Schema<Query, Mutate, EmptySubscription>;
