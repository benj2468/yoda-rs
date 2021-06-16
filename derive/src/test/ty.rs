use crate::Wrapper;

use super::super::Type;
use syn::parse_quote;

#[test]
fn ty_vec() {
    let syn_ty: syn::Type = parse_quote! {
        Vec<atoms::Identifier>
    };

    let ty = Type::from(syn_ty);

    assert_eq!(ty.wrapper, Wrapper::Vec);
    assert_eq!(ty.ty, parse_quote! { atoms::Identifier });
}

#[test]
fn ty_option() {
    let syn_ty: syn::Type = parse_quote! {
        Option<atoms::Identifier>
    };

    let ty = Type::from(syn_ty);

    assert_eq!(ty.wrapper, Wrapper::Option);
    assert_eq!(ty.ty, parse_quote! { atoms::Identifier });
}

#[test]
fn ty_option_w_inner() {
    let syn_ty: syn::Type = parse_quote! {
        Option<DateTime<Utc>>
    };

    let ty = Type::from(syn_ty);

    assert_eq!(ty.wrapper, Wrapper::Option);
    assert_eq!(ty.ty, parse_quote! { DateTime<Utc> });
}

#[test]
fn ty_get_out_str() {
    let syn_ty: syn::Type = parse_quote! {
        Option<DateTime<Utc>>
    };

    let ty = Type::from(syn_ty);

    assert_eq!(ty.ty_str(), "DateTime");
}
