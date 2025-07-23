# reqwest-sse

[![Made With Rust][made-with-rust]][rust]
[![Crates.io][badge-crates.io]][reqwest-sse-crates.io]
[![Docs.rs][badge-docs.rs]][reqwest-sse-docs.rs]

`reqwest-sse` is a lightweight Rust library that extends
[reqwest](https://docs.rs/reqwest) by adding native support for handling
[Server-Sent Events (SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
. It introduces the `EventSource` trait, which enhances reqwest's `Response`
type with an ergonomic `.events()` method. This method transforms the
response body into an asynchronous stream of SSE events, enabling seamless
integration of real-time event handling in applications using the familiar
reqwest HTTP client.

> :warning: This library is experimental and **shouldn't be used in production**.

## Example

```rust
use reqwest_sse::EventSource;
use tokio_stream::StreamExt;

let events = reqwest::get("https://example.com/events")
    .await?
    .events()
    .await?
    .collect::<Vec<_>>();
```

[rust]: https://www.rust-lang.org/
[made-with-rust]: https://img.shields.io/badge/rust-f04041?style=for-the-badge&labelColor=c0282d&logo=rust 'Made With Rust'
[badge-crates.io]: https://img.shields.io/badge/crates.io-v0.1.0-orange.svg?style=for-the-badge 'View on crates.rs'
[reqwest-sse-crates.io]: https://crates.io/crates/reqwest-sse
[badge-docs.rs]: https://img.shields.io/badge/docs.rs-reqwest--sse-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs 'Read doc on docs.rs'
[reqwest-sse-docs.rs]: https://docs.rs/reqwest-sse
