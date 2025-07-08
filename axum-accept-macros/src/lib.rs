//! The proc-macro crate of axum-accept.
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]
extern crate proc_macro;

use mediatype::MediaTypeBuf;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Fields, GenericParam, Ident, Lit, Meta, TypeParam,
    TypeParamBound, parse_macro_input,
};

/// This is the proc macro for `AcceptExtractor`.
///
/// # Panics
///
/// If it fails to parse the attributes.
#[proc_macro_derive(AcceptExtractor, attributes(accept))]
pub fn derive_accept_extractor(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut generics = input.generics.clone();

    // we need to add <S: Send + Sync> to the impl generics for FromRequestParts
    let s_param = GenericParam::Type(TypeParam {
        attrs: vec![],
        ident: Ident::new("S", proc_macro2::Span::call_site()),
        colon_token: Some(syn::token::Colon::default()),
        bounds: {
            let mut bounds = syn::punctuated::Punctuated::new();
            bounds.push(TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: syn::parse_str("Send").unwrap(),
            }));
            bounds.push(TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: syn::parse_str("Sync").unwrap(),
            }));
            bounds
        },
        eq_token: None,
        default: None,
    });
    generics.params.push(s_param);

    let (impl_generics, _, _) = generics.split_for_impl();

    let Data::Enum(data) = &input.data else {
        panic!("AcceptExtractor can only be derived for enums");
    };

    let mut match_arms = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let mediatype_raw = get_accept_mediatype(&variant.attrs);
        let mediatype = MediaTypeBuf::from_string(mediatype_raw.clone()) // compile time so clone is fine
            .expect("Failed to parse mediatype");
        let (ty, subty, suffix) = (
            mediatype.ty().as_str(),
            mediatype.subty().as_str(),
            mediatype.suffix().map(|s| s.as_str()),
        );

        match &variant.fields {
            Fields::Unit => {
                // quote encodes None to empty string, so we need to take extra
                // steps
                if let Some(suffix) = suffix {
                    match_arms.push(quote! {
                        (#ty, #subty, Some(#suffix)) => return Ok(#name::#variant_name),
                    });
                } else {
                    match_arms.push(quote! {
                        (#ty, #subty, None) => return Ok(#name::#variant_name),
                    });
                }
            }
            _ => panic!("Only unit fields are supported"),
        }
    }

    let expanded = quote! {
        impl #impl_generics axum::extract::FromRequestParts<S> for #name #ty_generics #where_clause {
            type Rejection = axum_accept::AcceptRejection;

            async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
                for mt in axum_accept::parse_mediatypes(&parts.headers)? {
                    match (mt.ty.as_str(), mt.subty.as_str(), mt.suffix.map(|s| s.as_str())) {
                        #(#match_arms)*
                        _ => {} // continue searching
                    }
                }

                Err(axum_accept::AcceptRejection::NoSupportedMediaTypeFound)
            }
        }
    };

    TokenStream::from(expanded)
}

fn get_accept_mediatype(attrs: &[Attribute]) -> String {
    for attr in attrs {
        if attr.path().is_ident("accept") {
            if let Meta::List(meta_list) = &attr.meta {
                for nested in meta_list
                    .parse_args_with(
                        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
                    )
                    .expect("Failed to parse args")
                {
                    if let syn::Meta::NameValue(name_value) = nested {
                        if name_value.path.is_ident("mediatype") {
                            if let syn::Expr::Lit(expr_lit) = &name_value.value {
                                if let Lit::Str(lit_str) = &expr_lit.lit {
                                    return lit_str.value();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    panic!(r#"Missing #[accept(mediatype = "...")]"#)
}
