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
