# reqwest-sse

Small rust library to consume [reqwest](https://docs.rs/reqwest)'s responses as
[Server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
stream.

> :warning: This library is experimental and **shouldn't be used in production**.

## Example

```rust
let events = reqwest::get("https://sse.test-free.online/api/story")
    .await?
    .events()
    .await?
    .collect::<Vec<_>>();
```
