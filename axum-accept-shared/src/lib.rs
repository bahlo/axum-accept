//! This crate contains shared types and functions used by both axum-accept
//! and axum-accept-derive.
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]
use std::{cmp::Ordering, fmt::Display, str::FromStr};

use axum::{
    http::{HeaderMap, StatusCode, header::ToStrError},
    response::{IntoResponse, Response},
};
use mediatype::{MediaType, MediaTypeError, MediaTypeList, Name, ReadParams, names::_STAR};

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

impl Display for AcceptRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (_, message) = self.status_and_message();
        write!(f, "{message}")
    }
}

impl std::error::Error for AcceptRejection {}

/// Parse and process the media types from the accept header.
///
/// # Errors
///
/// Returns an error if the accept header is invalid or no match was found.
pub fn parse_mediatypes(headers: &HeaderMap) -> Result<Vec<MediaType<'_>>, AcceptRejection> {
    let accept_header = headers
        .get("accept")
        .map(|header| header.to_str())
        .transpose()
        .map_err(AcceptRejection::InvalidHeader)?
        .unwrap_or_default();

    let Some(q_name) = Name::new("q") else {
        unreachable!()
    };

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
        let ord = b_q.cmp(a_q);
        match ord {
            Ordering::Less | Ordering::Greater => ord,
            Ordering::Equal => {
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

                Ordering::Equal
            }
        }
    });

    Ok(list.into_iter().map(|(_, mt)| mt).collect())
}

#[cfg(test)]
mod tests {
    use super::{AcceptRejection, parse_mediatypes};
    use axum::http::HeaderMap;
    use mediatype::media_type;

    #[test]
    fn test_parse_mediatype_invisible_ascii() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "‎ ".parse().unwrap()); // invisible ascii is verboten
        match parse_mediatypes(&headers) {
            Err(AcceptRejection::InvalidHeader(_)) => {}
            _ => panic!("expected invalid header rejection"),
        }
    }

    #[test]
    fn test_parse_mediatype_invalid_media_type() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "lol".parse().unwrap());
        match parse_mediatypes(&headers) {
            Err(AcceptRejection::InvalidMediaType(i, _)) => assert_eq!(i, 0),
            _ => panic!("expected invalid media type rejection"),
        }
    }

    #[test]
    fn test_parse_mediatype_invalid_q() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain,application/json;q=lol".parse().unwrap(),
        );
        match parse_mediatypes(&headers) {
            Err(AcceptRejection::InvalidQ(i, _)) => assert_eq!(i, 1),
            _ => panic!("expected invalid q rejection"),
        }
    }

    #[test]
    fn test_parse_mediatype_valid_types() {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/plain".parse().unwrap());
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(vec![media_type!(TEXT / PLAIN)], list);

        let mut headers = HeaderMap::new();
        headers.insert("accept", "text/plain,application/json".parse().unwrap());
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(
            vec![media_type!(TEXT / PLAIN), media_type!(APPLICATION / JSON)],
            list
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain,application/json;q=0.9".parse().unwrap(),
        );
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(2, list.len());
        assert_eq!(media_type!(TEXT / PLAIN), list[0]);
        assert_eq!(media_type!(APPLICATION / JSON), list[1].essence());
    }

    #[test]
    fn test_parse_mediatype_order() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/plain;q=0.9,application/json".parse().unwrap(),
        );
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
        assert_eq!(2, list.len());
        assert_eq!(media_type!(APPLICATION / JSON), list[0]);
        assert_eq!(media_type!(TEXT / PLAIN), list[1].essence());

        let mut headers = HeaderMap::new();
        headers.insert(
            "accept",
            "text/*,text/plain,application/json".parse().unwrap(),
        );
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
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
        let list = parse_mediatypes(&headers).expect("Accept header should've parsed correctly");
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
