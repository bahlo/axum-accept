//! This library allows you to specify which media-types you accept in Axum in
//! a typed way.
//!
//! # Example
//!
//! ```rust
//! use axum::{extract::Json, response::{IntoResponse, Response}};
//! use axum_accept::{typed_media_type, Accept2};
//! use serde::Serialize;
//!
//! typed_media_type!(TextPlain: TEXT/PLAIN);
//! typed_media_type!(ApplicationJson: APPLICATION/JSON);
//!
//! #[derive(Debug, Serialize)]
//! struct Message {
//!     content: String,
//! }
//!
//! async fn my_handler(accept: Accept2<TextPlain, ApplicationJson>) -> Response {
//!     match accept {
//!         Accept2::A(TextPlain(_)) => "hello world".into_response(),
//!         Accept2::B(ApplicationJson(_)) => Json(Message { content: "hello_world".to_string() }).into_response(),
//!     }
//! }
//! ```
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]

use std::{cmp::Ordering, str::FromStr};

use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header::ToStrError, request::Parts},
    response::{IntoResponse, Response},
};
use mediatype::{MediaType, MediaTypeError, MediaTypeList, Name, ReadParams, names::_STAR};

#[doc(hidden)]
pub use mediatype;

/// This type is meant to be implemented for newtypes around
/// `mediatype::MediaType`, created with `typed_media_type`.
pub trait AssociatedMediaType {
    /// Construct this type. Will panic if it doesn't match the associated media
    /// type.
    fn new(media_type: mediatype::MediaType<'static>) -> Self;
    /// The media type associated with this type.
    fn associated_media_type() -> mediatype::MediaType<'static>;
}

/// Construct a new typed media type.
///
/// # Example
///
/// ```rust
/// use axum_accept::typed_media_type;
///
/// typed_media_type!(TextPlain: TEXT/PLAIN);
/// ```
#[macro_export]
macro_rules! typed_media_type {
    ($name:ident: $ty:ident/$subty:ident) => {
        pub struct $name($crate::mediatype::MediaType<'static>);

        impl $crate::AssociatedMediaType for $name {
            fn new(media_type: $crate::mediatype::MediaType<'static>) -> Self {
                if media_type != Self::associated_media_type() {
                    panic!("Attempted to create typed media type with non-matching inner value");
                }

                Self(media_type)
            }

            fn associated_media_type() -> $crate::mediatype::MediaType<'static> {
                $crate::mediatype::media_type!($ty / $subty)
            }
        }
    };
}

/// The error type returned in the `FromRequestParts` implementations.
#[derive(Debug)]
pub enum AcceptRejection {
    /// The header could not be converted to a &str.
    InvalidHeader(ToStrError),
    /// The media type at index .0 could not be parsed.
    InvalidMediaType(usize, MediaTypeError),
    /// Invalid q parameter
    InvalidQ(usize, <f64 as FromStr>::Err),
    /// No supported media type was found.
    NoSupportedMediaTypeFound,
}

impl AcceptRejection {
    /// Get the status and message for an error.
    #[must_use]
    pub fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            Self::InvalidHeader(e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid accept header: {e}"),
            ),
            Self::InvalidMediaType(i, e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid media type in accept header at index {i}: {e}"),
            ),
            Self::InvalidQ(i, e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid q parameter in accept header at index {i}: {e}"),
            ),
            Self::NoSupportedMediaTypeFound => (
                StatusCode::NOT_ACCEPTABLE,
                "Accept header does not contain supported media types".to_string(),
            ),
        }
    }
}

impl IntoResponse for AcceptRejection {
    fn into_response(self) -> Response {
        self.status_and_message().into_response()
    }
}

fn get_media_type_list(headers: &HeaderMap) -> Result<Vec<MediaType<'_>>, AcceptRejection> {
    let accept_header = headers
        .get("accept")
        .map(|header| header.to_str())
        .transpose()
        .map_err(AcceptRejection::InvalidHeader)?
        .unwrap_or_default();

    let q_name = Name::new("q").expect("Expected 'q' to be a valid name");
    let mut list = MediaTypeList::new(accept_header)
        .enumerate()
        .map(|(i, mt)| match mt {
            // validate q parameter and add it as u16 for sorting
            Ok(mt) => Ok(match mt.get_param(q_name) {
                Some(q_str) => {
                    let q: f64 = q_str
                        .as_str()
                        .parse::<f64>()
                        .map_err(|e| AcceptRejection::InvalidQ(i, e))?
                        .clamp(0.0, 1.0);

                    // q is clamped to 0.0-1.0 so nothing can happen here
                    #[allow(clippy::cast_possible_truncation)]
                    #[allow(clippy::cast_sign_loss)]
                    ((q * 1000.0) as u16, mt)
                }
                None => (1000, mt),
            }),
            Err(e) => Err(AcceptRejection::InvalidMediaType(i, e)),
        })
        .collect::<Result<Vec<(u16, MediaType)>, AcceptRejection>>()?;

    list.sort_by(|(a_q, a_mt), (b_q, b_mt)| {
        if a_q == b_q {
            // both have the same q, order by specificity

            // is one of them */*? these come last
            if (a_mt.ty, a_mt.subty) == (_STAR, _STAR) {
                return Ordering::Greater;
            } else if (b_mt.ty, b_mt.subty) == (_STAR, _STAR) {
                return Ordering::Less;
            }

            // now check the subtype
            if a_mt.subty != b_mt.subty {
                if a_mt.subty == _STAR {
                    return Ordering::Greater;
                } else if b_mt.subty == _STAR {
                    return Ordering::Less;
                }
            }
        }

        b_q.cmp(a_q)
    });

    Ok(list.into_iter().map(|(_, mt)| mt).collect())
}

/// Accept a single media type.
#[derive(Debug)]
pub struct Accept<T: AssociatedMediaType>(T);

impl<S, T> FromRequestParts<S> for Accept<T>
where
    S: Sized + Send + Sync,
    T: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_t = T::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_t {
                return Ok(Accept(T::new(media_type_t)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

/// Accept 2 media types.
#[derive(Debug)]
pub enum Accept2<A, B>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
{
    /// The first media type.
    A(A),
    /// The second media type.
    B(B),
}

impl<S, A, B> FromRequestParts<S> for Accept2<A, B>
where
    S: Sized + Send + Sync,
    A: AssociatedMediaType,
    B: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_a = A::associated_media_type();
        let media_type_b = B::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_a {
                return Ok(Accept2::A(A::new(media_type_a)));
            } else if mt == media_type_b {
                return Ok(Accept2::B(B::new(media_type_b)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

/// Accept 3 media types.
#[derive(Debug)]
pub enum Accept3<A, B, C>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
{
    /// The first media type.
    A(A),
    /// The second media type.
    B(B),
    /// The third media type.
    C(C),
}

impl<S, A, B, C> FromRequestParts<S> for Accept3<A, B, C>
where
    S: Sized + Send + Sync,
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_a = A::associated_media_type();
        let media_type_b = B::associated_media_type();
        let media_type_c = C::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_a {
                return Ok(Accept3::A(A::new(media_type_a)));
            } else if mt == media_type_b {
                return Ok(Accept3::B(B::new(media_type_b)));
            } else if mt == media_type_c {
                return Ok(Accept3::C(C::new(media_type_c)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

/// Accept 4 media types.
#[derive(Debug)]
pub enum Accept4<A, B, C, D>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
{
    /// The first media type.
    A(A),
    /// The second media type.
    B(B),
    /// The third media type.
    C(C),
    /// The fourth media type.
    D(D),
}

impl<S, A, B, C, D> FromRequestParts<S> for Accept4<A, B, C, D>
where
    S: Sized + Send + Sync,
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_a = A::associated_media_type();
        let media_type_b = B::associated_media_type();
        let media_type_c = C::associated_media_type();
        let media_type_d = D::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_a {
                return Ok(Accept4::A(A::new(media_type_a)));
            } else if mt == media_type_b {
                return Ok(Accept4::B(B::new(media_type_b)));
            } else if mt == media_type_c {
                return Ok(Accept4::C(C::new(media_type_c)));
            } else if mt == media_type_d {
                return Ok(Accept4::D(D::new(media_type_d)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

/// Accept 5 media types.
#[derive(Debug)]
pub enum Accept5<A, B, C, D, E>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
    E: AssociatedMediaType,
{
    /// The first media type.
    A(A),
    /// The second media type.
    B(B),
    /// The third media type.
    C(C),
    /// The fourth media type.
    D(D),
    /// The fifth media type.
    E(E),
}

impl<S, A, B, C, D, E> FromRequestParts<S> for Accept5<A, B, C, D, E>
where
    S: Sized + Send + Sync,
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
    E: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_a = A::associated_media_type();
        let media_type_b = B::associated_media_type();
        let media_type_c = C::associated_media_type();
        let media_type_d = D::associated_media_type();
        let media_type_e = E::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_a {
                return Ok(Accept5::A(A::new(media_type_a)));
            } else if mt == media_type_b {
                return Ok(Accept5::B(B::new(media_type_b)));
            } else if mt == media_type_c {
                return Ok(Accept5::C(C::new(media_type_c)));
            } else if mt == media_type_d {
                return Ok(Accept5::D(D::new(media_type_d)));
            } else if mt == media_type_e {
                return Ok(Accept5::E(E::new(media_type_e)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

/// Accept 6 media types.
#[derive(Debug)]
pub enum Accept6<A, B, C, D, E, F>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
    E: AssociatedMediaType,
    F: AssociatedMediaType,
{
    /// The first media type.
    A(A),
    /// The second media type.
    B(B),
    /// The third media type.
    C(C),
    /// The fourth media type.
    D(D),
    /// The fifth media type.
    E(E),
    /// The sixth media type.
    F(F),
}

impl<S, A, B, C, D, E, F> FromRequestParts<S> for Accept6<A, B, C, D, E, F>
where
    S: Sized + Send + Sync,
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
    E: AssociatedMediaType,
    F: AssociatedMediaType,
{
    type Rejection = AcceptRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let media_type_a = A::associated_media_type();
        let media_type_b = B::associated_media_type();
        let media_type_c = C::associated_media_type();
        let media_type_d = D::associated_media_type();
        let media_type_e = E::associated_media_type();
        let media_type_f = F::associated_media_type();
        for mt in get_media_type_list(&parts.headers)? {
            if mt == media_type_a {
                return Ok(Accept6::A(A::new(media_type_a)));
            } else if mt == media_type_b {
                return Ok(Accept6::B(B::new(media_type_b)));
            } else if mt == media_type_c {
                return Ok(Accept6::C(C::new(media_type_c)));
            } else if mt == media_type_d {
                return Ok(Accept6::D(D::new(media_type_d)));
            } else if mt == media_type_e {
                return Ok(Accept6::E(E::new(media_type_e)));
            } else if mt == media_type_f {
                return Ok(Accept6::F(F::new(media_type_f)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mediatype::media_type;

    #[test]
    fn test_get_media_type_list_invisible_ascii() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "‎ ".parse().unwrap()); // invisible ascii is verboten
        match get_media_type_list(&headers) {
            Err(AcceptRejection::InvalidHeader(_)) => {}
            _ => panic!("expected invalid header rejection"),
        }
    }

    #[test]
    fn test_get_media_type_list_invalid_media_type() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "lol".parse().unwrap());
        match get_media_type_list(&headers) {
            Err(AcceptRejection::InvalidMediaType(i, _)) => assert_eq!(i, 0),
            _ => panic!("expected invalid media type rejection"),
        }
    }

    #[test]
    fn test_get_media_type_list_invalid_q() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain,application/json;q=lol".parse().unwrap(),
        );
        match get_media_type_list(&headers) {
            Err(AcceptRejection::InvalidQ(i, _)) => assert_eq!(i, 1),
            _ => panic!("expected invalid q rejection"),
        }
    }

    #[test]
    fn test_get_media_type_list_valid_types() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/plain".parse().unwrap());
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(vec![media_type!(TEXT / PLAIN)], list);

        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/plain,application/json".parse().unwrap());
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(
            vec![media_type!(TEXT / PLAIN), media_type!(APPLICATION / JSON)],
            list
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain,application/json;q=0.9".parse().unwrap(),
        );
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(2, list.len());
        assert_eq!(media_type!(TEXT / PLAIN), list[0]);
        assert_eq!(media_type!(APPLICATION / JSON), list[1].essence());
    }

    #[test]
    fn test_get_media_type_list_order() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain;q=0.9,application/json".parse().unwrap(),
        );
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(2, list.len());
        assert_eq!(media_type!(APPLICATION / JSON), list[0]);
        assert_eq!(media_type!(TEXT / PLAIN), list[1].essence());

        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/*,text/plain,application/json".parse().unwrap(),
        );
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(
            vec![
                media_type!(TEXT / PLAIN),
                media_type!(APPLICATION / JSON),
                media_type!(TEXT / _STAR)
            ],
            list
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "*/*,text/*,text/plain,application/json".parse().unwrap(),
        );
        let list = get_media_type_list(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(
            vec![
                media_type!(TEXT / PLAIN),
                media_type!(APPLICATION / JSON),
                media_type!(TEXT / _STAR),
                media_type!(_STAR / _STAR)
            ],
            list
        );
    }
}
