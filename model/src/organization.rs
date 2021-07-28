use super::*;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Default, Clone, Debug, Api)]
#[auth(
    mutate = ["admin", "self"],
    query = ["admin", "organization", "user", "service"]
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
