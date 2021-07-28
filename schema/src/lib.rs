use async_graphql::EmptySubscription;
use model::{Mutate, Query};

pub type YodaSchema = async_graphql::Schema<Query, Mutate, EmptySubscription>;
