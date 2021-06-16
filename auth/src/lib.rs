use actix_web::{dev::ServiceRequest, error::Result, HttpMessage};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use jwks_client::error::Error;
use jwks_client::keyset::KeyStore;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq)]
pub enum Action {
    ReadOrganization,
    UpdateOrganization,
    CreateOrganization,
    All,
}

#[derive(Clone, PartialEq, Eq)]
pub enum Role {
    User,
    Organization,
    Admin,
}

#[derive(Clone)]
pub struct Identity {
    pub user_id: String,
    pub roles: Vec<Role>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    iss: String,
    sub: String,
    exp: usize,
}

impl Identity {
    pub async fn from_token(_token: &str) -> Result<Self> {
        // let authority = std::env::var("AUTHORITY").expect("AUTHORITY must be set");

        // let key_set = KeyStore::new_from(&format!(
        //     "{}{}",
        //     authority.as_str(),
        //     ".well-known/jwks.json"
        // ))
        // .await
        // .unwrap();

        // let jwt = key_set
        //     .verify(token)
        //     .map_err(|_| actix_web::error::ErrorForbidden("Could not verify JWT"))?;

        // let payload = jwt.payload();

        let roles = vec![Role::Admin];

        // Ok(Self {
        //     roles: roles,
        //     user_id: payload.sub().map(|e| e.into()).unwrap_or_default(),
        // })

        Ok(Self {
            roles,
            user_id: "fake_id".into(),
        })
    }

    pub fn is_authorized(&self, action: Action) -> async_graphql::Result<()> {
        let permitted_actions = self
            .roles
            .iter()
            .flat_map(|role| match role {
                Role::User => vec![Action::ReadOrganization],
                Role::Admin => vec![Action::All],
                Role::Organization => vec![Action::ReadOrganization],
            })
            .collect::<Vec<_>>();

        (permitted_actions.contains(&action) || permitted_actions.contains(&Action::All))
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
