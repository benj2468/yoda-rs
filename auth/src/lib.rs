use actix_web::{dev::ServiceRequest, error::Result, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

mod role;
pub use role::Role;

#[derive(Clone, PartialEq, Eq)]
pub enum Action {
    Query,
    Update,
    Create,
    All,
}

#[derive(Clone)]
pub struct Identity {
    pub user_id: String,
    pub roles: Vec<Role>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    app_metadata: AppMetaData,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppMetaData {
    roles: Vec<String>,
}

impl Identity {
    pub async fn from_token(token: &str) -> Result<Self> {
        let secret = std::env::var("JWK").map_err(actix_web::error::ErrorExpectationFailed)?;

        let token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(actix_web::error::ErrorForbidden)?;

        let roles: Vec<_> = token
            .claims
            .app_metadata
            .roles
            .iter()
            .filter_map(|role| Role::try_from(role).ok())
            .collect();

        if roles.is_empty() {
            return Err(actix_web::error::ErrorForbidden(
                "Must provide at least one role",
            ));
        }

        Ok(Self {
            roles,
            user_id: token.claims.sub,
        })
    }

    pub fn is_authorized(&self, permissive_roles: Vec<Role>) -> async_graphql::Result<()> {
        permissive_roles
            .into_iter()
            .any(|role| self.roles.contains(&role))
            .then(|| ())
            .ok_or_else(|| {
                async_graphql::Error::new("Your identitiy is not authorized to perform that action")
            })
    }
}

pub struct Validator;

impl Validator {
    pub async fn middleware(
        req: ServiceRequest,
        credentials: BearerAuth,
    ) -> Result<ServiceRequest> {
        let token = credentials.token();

        let identity = Identity::from_token(token).await?;

        req.extensions_mut().insert(identity);

        Ok(req)
    }
}
