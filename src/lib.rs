use axum::{
    extract::FromRequestParts,
    http::{HeaderMap, StatusCode, header::ToStrError, request::Parts},
    response::{IntoResponse, Response},
};
use mediatype::{MediaTypeError, MediaTypeList};

#[doc(hidden)]
pub use mediatype;

pub trait AssociatedMediaType {
    fn new(media_type: mediatype::MediaType<'static>) -> Self;
    fn associated_media_type() -> mediatype::MediaType<'static>;
}

#[macro_export]
macro_rules! typed_media_type {
    ($name:ident: $ty:ident/$subty:ident) => {
        pub struct $name($crate::mediatype::MediaType<'static>);

        impl AssociatedMediaType for $name {
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

#[derive(Debug)]
pub enum AcceptRejection {
    InvalidHeader(ToStrError),
    InvalidMediaType(usize, MediaTypeError),
    NoSupportedMediaTypeFound,
}

impl IntoResponse for AcceptRejection {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidHeader(e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid accept header: {e}"),
            ),
            Self::InvalidMediaType(i, e) => (
                StatusCode::BAD_REQUEST,
                format!("Invalid media type in accept header at index {i}: {e}"),
            ),
            Self::NoSupportedMediaTypeFound => (
                StatusCode::NOT_ACCEPTABLE,
                format!("Accept header does not contain supported media types"),
            ),
        }
        .into_response()
    }
}

fn get_media_type_list(headers: &HeaderMap) -> Result<MediaTypeList, AcceptRejection> {
    let accept_header = headers
        .get("accept")
        .map(|header| header.to_str())
        .transpose()
        .map_err(|e| AcceptRejection::InvalidHeader(e))?
        .unwrap_or_default();
    Ok(MediaTypeList::new(accept_header))
}

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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

            if mt == media_type_t {
                return Ok(Accept(T::new(media_type_t)));
            }

            // continue searching
        }

        Err(AcceptRejection::NoSupportedMediaTypeFound)
    }
}

#[derive(Debug)]
pub enum Accept2<A, B>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
{
    A(A),
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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

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

#[derive(Debug)]
pub enum Accept3<A, B, C>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
{
    A(A),
    B(B),
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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

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

#[derive(Debug)]
pub enum Accept4<A, B, C, D>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
{
    A(A),
    B(B),
    C(C),
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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

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

#[derive(Debug)]
pub enum Accept5<A, B, C, D, E>
where
    A: AssociatedMediaType,
    B: AssociatedMediaType,
    C: AssociatedMediaType,
    D: AssociatedMediaType,
    E: AssociatedMediaType,
{
    A(A),
    B(B),
    C(C),
    D(D),
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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

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
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
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
        for (i, mt) in get_media_type_list(&parts.headers)?.enumerate() {
            let mt = match mt {
                Ok(mt) => mt,
                Err(e) => {
                    return Err(AcceptRejection::InvalidMediaType(i, e));
                }
            };

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
mod tests {}
