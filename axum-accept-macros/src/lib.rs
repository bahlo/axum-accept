//! The proc-macro crate of axum-accept.
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]
extern crate proc_macro;

use std::collections::HashMap;

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
#[allow(clippy::too_many_lines)]
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
                path: syn::parse_str("Send").expect("Failed to parse 'Send'"),
            }));
            bounds.push(TypeParamBound::Trait(syn::TraitBound {
                paren_token: None,
                modifier: syn::TraitBoundModifier::None,
                lifetimes: None,
                path: syn::parse_str("Sync").expect("Failed to parse 'Sync'"),
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

    let has_default = data.variants.iter().any(|variant| {
        variant.attrs.iter().any(|attr| match &attr.meta {
            Meta::Path(path) => path.is_ident("default"),
            _ => false,
        })
    });

    // Match arms with ty, subty and suffix
    let mut match_arms = Vec::new();
    // Match arms with ty only (for checking mediatypes like text/*)
    let mut match_arms_tys = HashMap::new();
    // Store first variant to fall back to if we don't have a default.
    let mut first_variant_name = None;

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

        assert!(ty != "*" && subty != "*", "Please use a concrete mediatype");

        if first_variant_name.is_none() {
            first_variant_name = Some(variant_name.clone());
        }

        match_arms_tys.insert(ty.to_string(), variant_name);

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

    let check_and_return_default = if has_default {
        Some(quote! {
            if mediatypes.is_empty() {
                return Ok(#name::default());
            }
        })
    } else {
        None
    };

    let handle_star_star = if has_default {
        quote! {
            return Ok(#name::default());
        }
    } else {
        quote! {
            return Ok(#name::#first_variant_name);
        }
    };

    let match_arms_tys = match_arms_tys.iter().map(|(ty, variant_name)| {
        quote! {
            (#ty) => return Ok(#name::#variant_name),
        }
    });

    let expanded = quote! {
        impl #impl_generics axum::extract::FromRequestParts<S> for #name #ty_generics #where_clause {
            type Rejection = axum_accept::AcceptRejection;

            async fn from_request_parts(parts: &mut axum::http::request::Parts, _state: &S) -> Result<Self, Self::Rejection> {
                let mediatypes = axum_accept::parse_mediatypes(&parts.headers)?;
                #check_and_return_default
                for mt in mediatypes {
                    match (mt.ty.as_str(), mt.subty.as_str()) {
                        ("*", "*") => {
                            // return either the the default or the first
                            // variant
                            #handle_star_star
                        },
                        // do we have any mediatype that shares the main type?
                        // e.g. we offer text/plain and get accept: text/*
                        (_, "*") => match (mt.ty.as_str()) {
                            #(#match_arms_tys)*
                            _ => {} // continue searching
                        },
                        // do proper matching
                        _ =>  match (mt.ty.as_str(), mt.subty.as_str(), mt.suffix.map(|s| s.as_str())) {
                            #(#match_arms)*
                            _ => {} // continue searching
                        },
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
