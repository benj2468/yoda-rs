use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, TokenStreamExt};
use std::convert::TryFrom;

#[derive(Clone, PartialEq, Eq, derive_more::Display, Debug)]
pub enum Role {
    User,
    Organization,
    Admin,
    Service,

    // Inner facing role
    Own,
}

impl TryFrom<&String> for Role {
    type Error = String;
    fn try_from(input: &String) -> Result<Self, Self::Error> {
        match input.as_str() {
            "user" => Ok(Self::User),
            "organization" => Ok(Self::Organization),
            "admin" => Ok(Self::Admin),
            "service" => Ok(Self::Service),
            "self" => Ok(Self::Own),
            _ => Err(format!(
                "No Role specified for that string: {}",
                input.as_str()
            )),
        }
    }
}

impl ToTokens for Role {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.append(Ident::new("auth", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(Ident::new("Role", Span::call_site()));
        tokens.append(Punct::new(':', Spacing::Joint));
        tokens.append(Punct::new(':', Spacing::Alone));
        tokens.append(Ident::new(self.to_string().as_str(), Span::call_site()));
    }
}
