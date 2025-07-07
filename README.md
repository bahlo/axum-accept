# axum-accept

[![CI](https://github.com/bahlo/axum-accept/actions/workflows/ci.yml/badge.svg)](https://github.com/bahlo/axum-accept/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/axum-accept.svg)](https://crates.io/crates/axum-accept)
[![docs.rs](https://docs.rs/axum-accept/badge.svg)](https://docs.rs/axum-accept/)
[![License](https://img.shields.io/crates/l/axum-accept)](LICENSE-APACHE)

Typed accept negotiation for axum, following [RFC7231](https://www.rfc-editor.org/rfc/rfc7231).

## Example

```rust
use axum::{extract::Json, response::{IntoResponse, Response}};
use axum_accept::{typed_media_type, Accept2};
use serde::Serialize;

typed_media_type!(TextPlain: TEXT/PLAIN);
typed_media_type!(ApplicationJson: APPLICATION/JSON);

#[derive(Debug, Serialize)]
struct Message {
    content: String,
}

async fn my_handler(accept: Accept2<TextPlain, ApplicationJson>) -> Response {
    match accept {
        Accept2::A(TextPlain(_)) => "hello world".into_response(),
        Accept2::B(ApplicationJson(_)) => Json(Message { content: "hello_world".to_string() }).into_response(),
    }
}
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
