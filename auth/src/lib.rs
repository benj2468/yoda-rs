use actix_web::{dev::ServiceRequest, error::Result, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

mod role;
pub use role::Role;

#[derive(Clone, PartialEq, Eq)]
pub enum Action<'a> {
    Query,
    Mutate(&'a str),
    Create,
    All,
}

#[derive(Clone)]
pub struct Identity {
    pub user_id: String,
    pub roles: Vec<Role>,
}

/// # JWT Format
/// {
///     sub: id,
///     exp: epoch in seconds,
///     app_metadata: {
///         roles: Vec<"user" | "organization" | "service" ...>
///     }
/// }
///
/// ## For an Account
/// The `sub` should be the primary UUID for the user
///
/// ## For an Organization
/// The `sub` should be the primary UUID for the organization

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

    pub fn is_authorized(
        &self,
        action: Action,
        permissive_roles: Vec<Role>,
    ) -> async_graphql::Result<()> {
        let permitted = self.roles.iter().any(|role| match role {
            Role::Admin | Role::Service | Role::Organization => permissive_roles.contains(role),
            Role::User => match action {
                Action::Mutate(id) => {
                    if permissive_roles.contains(&Role::Own) {
                        id == self.user_id
                    } else {
                        true
                    }
                }
                _ => true,
            },
            Role::Own => false,
        });

        permitted.then(|| ()).ok_or_else(|| {
            async_graphql::Error::new("Your identity is not authorized to perform that action")
        })
    }
}

pub struct Validator;

impl Validator {
    pub async fn middleware(
        req: ServiceRequest,
        credentials: BearerAuth,
    ) -> Result<ServiceRequest> {
        let identity = {
            let token = credentials.token();

            Identity::from_token(token).await?
        };

        req.extensions_mut().insert(identity);

        Ok(req)
    }
}
