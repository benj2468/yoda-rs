use async_graphql::{EmptySubscription, MergedObject};
use org_service::{OrgMutate, OrgQuery};

#[derive(Default, MergedObject)]
pub struct Query(OrgQuery);

#[derive(Default, MergedObject)]
pub struct Mutate(OrgMutate);

pub type YodaSchema = async_graphql::Schema<Query, Mutate, EmptySubscription>;
