use async_graphql::{Enum, InputObject, SimpleObject};
use derive_more::Display;

pub mod delta;
pub mod pagination;
pub mod search;

#[derive(SimpleObject, serde::Serialize, serde::Deserialize, Hash, Clone, Debug, PartialEq)]
pub struct Identifier {
    pub value: String,
    pub system: IdentifierSystem,
    tier: IdentifierTier,
}

impl Identifier {
    pub fn is_primary(&self) -> bool {
        matches!(self.tier, IdentifierTier::Primary)
    }

    pub fn new(system: IdentifierSystem, tier: IdentifierTier) -> Self {
        Self {
            system,
            tier,
            value: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl ToString for Identifier {
    fn to_string(&self) -> String {
        format!("{}/{}", self.system, self.value)
    }
}

#[derive(
    Enum, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash, Display, Debug,
)]
pub enum IdentifierTier {
    Primary,
    Secondary,
    Other,
}

#[derive(
    Enum, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash, Display, Debug,
)]
pub enum IdentifierSystem {
    Yoda,
    Other,
}

#[derive(InputObject)]
pub struct IdentifierInput {
    pub value: String,
    pub system: IdentifierSystem,
    pub tier: IdentifierTier,
}

impl From<IdentifierInput> for String {
    fn from(input: IdentifierInput) -> Self {
        format!("{}/{}", input.system, input.value)
    }
}

impl From<IdentifierInput> for Identifier {
    fn from(input: IdentifierInput) -> Self {
        Self {
            system: match input.system.to_string().as_str() {
                "Yoda" => IdentifierSystem::Yoda,
                _ => IdentifierSystem::Other,
            },
            value: input.value,
            tier: input.tier,
        }
    }
}

#[derive(Enum, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize, Hash)]
pub enum Tag {
    Religious,
    Education,
    Politics,
}
