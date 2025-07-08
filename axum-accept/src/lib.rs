//! Typed accept negotiation for axum, following RFC7231.
//!
//! # Example
//!
//! ```rust
//! use axum_accept::AcceptExtractor;
//!
//! #[derive(AcceptExtractor)]
//! enum Accept {
//!     #[accept(mediatype="text/plain")]
//!     TextPlain,
//! }
//! ```
#![deny(warnings)]
#![deny(clippy::pedantic, clippy::unwrap_used)]
#![deny(missing_docs)]
pub use axum_accept_macros::AcceptExtractor;
pub use axum_accept_shared::AcceptRejection;

#[doc(hidden)]
pub use axum_accept_shared::parse_mediatypes;

#[cfg(doctest)]
#[doc = include_str!("../../README.md")]
pub struct ReadmeDoctests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate as axum_accept; // necessary for the macro to work
    use axum::{
        body::Body,
        extract::{FromRequest, Request},
    };

    #[derive(Debug, AcceptExtractor)]
    enum Accept {
        #[accept(mediatype = "text/plain")]
        TextPlain,
        #[accept(mediatype = "application/json")]
        ApplicationJson,
        #[accept(mediatype = "application/ld+json")]
        ApplicationLdJson,
    }

    #[tokio::test]
    async fn test_accept_extractor_basic() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "application/json,text/plain")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state)
            .await
            .expect("Expected no rejection");
        let Accept::ApplicationJson = media_type else {
            panic!("expected application/json")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_q() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "application/json;q=0.9,text/plain")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state)
            .await
            .expect("Expected no rejection");
        let Accept::TextPlain = media_type else {
            panic!("expected text/plain")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_specifity() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/*,text/plain")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state)
            .await
            .expect("Expected no rejection");
        let Accept::TextPlain = media_type else {
            panic!("expected text/plain")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_suffix() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/*,application/ld+json,text/plain")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state)
            .await
            .expect("Expected no rejection");
        let Accept::ApplicationLdJson = media_type else {
            panic!("expected application/ldjson")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_no_match() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/csv")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state).await;
        let Err(AcceptRejection::NoSupportedMediaTypeFound) = media_type else {
            panic!("expected no supported media type found")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_star() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/csv,text/*")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state).await;
        let Ok(Accept::TextPlain) = media_type else {
            panic!("expected text/*, got {:?}", media_type)
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_star_star() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/csv,*/*")
            .body(Body::from(""))?;
        let state = ();
        let media_type = Accept::from_request(req, &state).await;
        let Ok(Accept::TextPlain) = media_type else {
            panic!("expected text/plain")
        };
        Ok(())
    }

    #[derive(Debug, AcceptExtractor, Default)]
    enum AcceptWithDefault {
        #[accept(mediatype = "application/json")]
        ApplicationJson,
        #[default]
        #[accept(mediatype = "text/plain")]
        TextPlain,
    }

    #[tokio::test]
    async fn test_accept_extractor_default() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/csv")
            .body(Body::from(""))?;
        let state = ();
        let media_type = AcceptWithDefault::from_request(req, &state).await;
        let Err(AcceptRejection::NoSupportedMediaTypeFound) = media_type else {
            panic!("expected no supported media type found")
        };

        let req = Request::builder().body(Body::from(""))?;
        let state = ();
        let media_type = AcceptWithDefault::from_request(req, &state).await;
        let Ok(AcceptWithDefault::TextPlain) = media_type else {
            panic!("expected text/plain (default)")
        };
        Ok(())
    }

    #[tokio::test]
    async fn test_accept_extractor_star_star_default() -> Result<(), Box<dyn std::error::Error>> {
        let req = Request::builder()
            .header("accept", "text/csv,*/*")
            .body(Body::from(""))?;
        let state = ();
        let media_type = AcceptWithDefault::from_request(req, &state).await;
        let Ok(AcceptWithDefault::TextPlain) = media_type else {
            panic!("expected text/plain (default)")
        };
        Ok(())
    }
}
