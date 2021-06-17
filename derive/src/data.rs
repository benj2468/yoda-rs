use self::attribute::auth::AuthAttribute;
use super::*;
use std::convert::{TryFrom, TryInto};

pub(crate) struct DeriveData {
    pub(crate) ident: Ident,
    #[allow(dead_code)]
    pub(crate) vis: Visibility,
    pub(crate) fields: Vec<Field>,
    pub(crate) attributes: Vec<Attribute>,
}

impl From<TokenStream> for DeriveData {
    fn from(tokens: TokenStream) -> Self {
        let DeriveInput {
            attrs,
            data,
            ident,
            vis,
            ..
        } = syn::parse(tokens).unwrap();

        let fields: Vec<Field> = match data {
            Data::Struct(data) => data.fields.into_iter().map(|field| field.into()).collect(),
            _ => unimplemented!("DeriveData is only for a struct"),
        };

        Self {
            attributes: attrs
                .into_iter()
                .filter_map(|e| e.try_into().ok())
                .collect(),
            fields,
            ident,
            vis,
        }
    }
}

impl DeriveData {
    pub fn auth_attribute(&self) -> AuthAttribute {
        self.attributes
            .iter()
            .find_map(|attr| {
                if let Attribute::Auth(attr) = attr {
                    Some(attr)
                } else {
                    None
                }
            })
            .cloned()
            .unwrap_or_default()
    }
}

pub(crate) struct Field {
    pub(crate) ident: Ident,
    #[allow(dead_code)]
    pub(crate) vis: Visibility,
    pub(crate) ty: Type,
    pub(crate) attributes: Vec<Attribute>,
}

impl Field {
    pub fn is_identifier(&self) -> bool {
        self.ident == "identifier"
    }
    pub fn wrapped(&self) -> TokenStream2 {
        let ty = &self.ty.ty;
        match self.ty.wrapper {
            Wrapper::Option => {
                quote! { Option<#ty> }
            }
            Wrapper::Vec => {
                quote! { Vec<#ty> }
            }
        }
    }
}

impl From<syn::Field> for Field {
    fn from(f: syn::Field) -> Self {
        let ident = f.ident.expect("Ident was not provided for a field");

        let ty = Type::from(f.ty);

        Self {
            ident,
            vis: f.vis,
            ty,
            attributes: f
                .attrs
                .into_iter()
                .filter_map(|attr| attr.try_into().ok())
                .collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Type {
    pub(crate) wrapper: Wrapper,
    pub(crate) ty: syn::Type,
}

impl From<syn::Type> for Type {
    fn from(ty: syn::Type) -> Self {
        match ty {
            syn::Type::Path(type_path) => {
                let seg = type_path
                    .path
                    .segments
                    .last()
                    .expect("Must have at least some type");

                let ty = get_inner_ty(seg);

                let wrapper = match seg.ident.to_string().as_str() {
                    "Option" => Wrapper::Option,
                    "Vec" => Wrapper::Vec,
                    _ => unimplemented!("Type must be Vec or option wrapped"),
                };

                Self { wrapper, ty }
            }
            _ => unimplemented!("Only path types are supported"),
        }
    }
}

impl Type {
    pub fn ty_str(&self) -> String {
        let ty = &self.ty;

        match ty {
            syn::Type::Path(path) => path
                .path
                .segments
                .last()
                .expect("must have at least one segment")
                .ident
                .to_string(),
            _ => unimplemented!("Only path types"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Wrapper {
    Option,
    Vec,
}

fn get_inner_ty(origin: &PathSegment) -> syn::Type {
    match origin.arguments.clone() {
        PathArguments::AngleBracketed(bracket) => {
            let arg = bracket
                .args
                .into_iter()
                .last()
                .expect("Must have one argument");

            match arg {
                GenericArgument::Type(ty) => ty,
                _ => unimplemented!("Only type arguments supported"),
            }
        }
        _ => unimplemented!("Only Angle Bracketed paths supported"),
    }
}

mod attribute;

#[derive(PartialEq, Eq, Debug)]
pub(crate) enum Attribute {
    Struct,
    Auth(attribute::auth::AuthAttribute),
    Search,
    Doc,
}

impl TryFrom<syn::Attribute> for Attribute {
    type Error = ();

    fn try_from(attr: syn::Attribute) -> Result<Self, Self::Error> {
        if let Some(seg) = attr.path.segments.last() {
            match seg.ident.to_string().as_str() {
                "construct" => Ok(Self::Struct),
                "auth" => Ok(Self::Auth(
                    attr.parse_args().unwrap_or_else(|e| panic!("{}", e)),
                )),
                "searchable" => Ok(Self::Search),
                "doc" => Ok(Self::Doc),
                _ => Err(()),
            }
        } else {
            Err(())
        }
    }
}
