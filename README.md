# axum-accept

[![CI](https://github.com/bahlo/axum-accept/actions/workflows/ci.yml/badge.svg)](https://github.com/bahlo/axum-accept/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/axum-accept.svg)](https://crates.io/crates/axum-accept)
[![docs.rs](https://docs.rs/axum-accept/badge.svg)](https://docs.rs/axum-accept/)
[![License](https://img.shields.io/crates/l/axum-accept)](LICENSE-APACHE)

Typed accept negotiation for axum, following [RFC7231](https://www.rfc-editor.org/rfc/rfc7231).

## Example

```rust
use axum::{extract::Json, response::{IntoResponse, Response}};
use axum_accept::AcceptExtractor;
use serde_json::json;

#[derive(AcceptExtractor, Default)]
enum Accept {
    #[accept(mediatype="text/plain")]
    TextPlain,
    #[default]
    #[accept(mediatype="application/json")]
    ApplicationJson,
}

async fn my_handler(accept: Accept) -> Response {
    match accept {
        Accept::TextPlain => "hello world".into_response(),
        Accept::ApplicationJson => Json(json!({ "content": "hello_world" })).into_response(),
    }
}
```

## Edge cases

Setting a default is recommended as it indicates behaviour more explicitly in
your code.
This is how axum-accept behaves on edge cases:

| Accept    | Has default               | No default                |
| --------- | ------------------------- | ------------------------- |
| `<empty>` | Default variant           | HTTP 406 (Not Acceptable) |
| `*/*`     | Default variant           | First variant             |
 
## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
