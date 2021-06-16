use actix_web::{dev::ServiceRequest, error::Result, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

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
        let secret =
            std::env::var("JWK").map_err(|e| actix_web::error::ErrorExpectationFailed(e))?;

        let token = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| actix_web::error::ErrorForbidden(e))?;

        let roles = token
            .claims
            .app_metadata
            .roles
            .iter()
            .map(|role| match role.as_str() {
                "admin" => Ok(Role::Admin),
                "user" => Ok(Role::User),
                "organization" => Ok(Role::Organization),
                _ => Err(actix_web::error::ErrorUnauthorized(
                    "Must provide at least some role",
                )),
            })
            .collect::<Result<Vec<Role>>>()?;

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
