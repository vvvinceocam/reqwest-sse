# reqwest-sse

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
