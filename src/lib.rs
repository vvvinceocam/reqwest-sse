//!  `reqwest-sse` is a lightweight Rust library that extends
//! [reqwest](https://docs.rs/reqwest) by adding native support of
//! [Server-Sent Events (SSE)](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events)
//! . It introduces the [`EventSource`] trait, which extends reqwest's [`Response`]
//! type with an ergonomic `.events()` method. This method transforms the
//! response body into an asynchronous [Stream] of SSE [`Event`]s, enabling
//! seamless integration of real-time event handling in applications
//! using the familiar reqwest HTTP client and the [`StreamExt`] API.
//!
//! ## Example
//!
//! ```rust,no_run
//! use tokio_stream::StreamExt;
//!
//! use reqwest_sse::EventSource;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut events = reqwest::get("https://sse.test-free.online/api/story")
//!         .await.unwrap()
//!         .events()
//!         .await.unwrap();
//!
//!     while let Some(Ok(event)) = events.next().await {
//!         println!("{event:?}");
//!     }
//! }
//! ```
pub mod error;

use std::{pin::Pin, time::Duration};

use async_stream::try_stream;
use bytes::BytesMut;
use reqwest::{
    Response, StatusCode,
    header::{CONTENT_TYPE, HeaderValue},
};
use tokio_stream::{Stream, StreamExt};
use tokio_util::{
    codec::{Decoder, FramedRead},
    io::StreamReader,
};

use crate::error::{EventError, EventSourceError};

/// Represents a stream of Server-Sent Events.
pub type ServerSentEvents = Pin<Box<dyn Stream<Item = Result<Event, EventError>>>>;

pub static MIME_EVENT_STREAM: &[u8] = b"text/event-stream";

/// Returns `true` if the given [`HeaderValue`] is an event stream MIME type.
fn is_event_stream(value: &HeaderValue) -> bool {
    value
        .as_bytes()
        .split(|&b| b == b';')
        .next()
        .unwrap_or(b"")
        .trim_ascii()
        .eq_ignore_ascii_case(MIME_EVENT_STREAM)
}

/// Adapts [`russe::Decoder`] to discard an incomplete SSE event at EOF.
///
/// [`FramedRead`] calls [`Decoder::decode_eof`] when the response ends. Its default
/// implementation reports an error if a partial frame remains. This wrapper delegates
/// parsing to `russe` and lets the stream end after the last complete event.
#[derive(Default)]
struct EventDecoder(russe::Decoder);

impl Decoder for EventDecoder {
    type Item = russe::Event;
    type Error = russe::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> russe::Result<Option<Self::Item>> {
        self.0.decode(buf)
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> russe::Result<Option<Self::Item>> {
        self.decode(buf)
    }
}

/// Server-Sent Event representation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    /// A string identifying the type of event described.
    pub event_type: String,
    /// The data field for the message.
    pub data: String,
    /// Last event ID value.
    pub last_event_id: Option<String>,
    /// Reconnection time.
    pub retry: Option<Duration>,
}

/// A trait for consuming a [`Response`] as a [`Stream`] of Server-Sent [`Event`]s (SSE).
pub trait EventSource {
    /// Converts the [`Response`] into a stream of Server-Sent Events.
    /// Returns it as a faillable [`Stream`] of [`Event`]s.
    ///
    /// # Errors
    ///
    /// Returns an [`EventSourceError`] if:
    /// - The response status is not `200 OK`
    /// - The `Content-Type` header is missing or not `text/event-stream`
    ///
    /// The stream yields an [`EventError`] when error occure on event reading.
    fn events(self) -> impl Future<Output = Result<ServerSentEvents, EventSourceError>> + Send;
}

impl EventSource for Response {
    // Keep `async` so the `!Send` `ServerSentEvents` stream is built on poll,
    // allowing the returned future to stay `Send`.
    #[allow(clippy::unused_async_trait_impl)]
    async fn events(self) -> Result<ServerSentEvents, EventSourceError> {
        let status = self.status();
        if status != StatusCode::OK {
            return Err(EventSourceError::BadStatus(status));
        }
        match self.headers().get(CONTENT_TYPE) {
            Some(content_type) => {
                if !is_event_stream(content_type) {
                    return Err(EventSourceError::BadContentType(Some(
                        content_type.to_owned(),
                    )));
                }
            }
            None => return Err(EventSourceError::BadContentType(None)),
        }

        let body = self
            .bytes_stream()
            .map(|chunk| chunk.map_err(std::io::Error::other));
        let mut frames = FramedRead::new(StreamReader::new(body), EventDecoder::default());

        let stream = Box::pin(try_stream! {
            let mut last_event_id = None;
            let mut retry = None;

            while let Some(frame) = frames.next().await {
                match frame.map_err(EventError::from_decoder)? {
                    russe::Event::Comment(_) => {}
                    russe::Event::Retry(delay) => retry = Some(delay),
                    russe::Event::Message(message) => {
                        if let Some(id) = message.id {
                            last_event_id = Some(id.to_string());
                        }
                        retry = message.retry.or(retry);

                        if !message.data.is_empty() {
                            yield Event {
                                event_type: message.event.as_deref()
                                    .filter(|name| !name.is_empty())
                                    .unwrap_or("message")
                                    .to_owned(),
                                data: message.data.to_string(),
                                last_event_id: last_event_id.clone(),
                                retry,
                            };
                        }
                    }
                }
            }
        });

        Ok(stream)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_event_stream_accept_valid_values() {
        assert!(is_event_stream(&HeaderValue::from_static(
            "text/event-stream"
        )));
        assert!(is_event_stream(&HeaderValue::from_static(
            "text/event-stream; charset=utf-8"
        )));
        assert!(is_event_stream(&HeaderValue::from_static(
            "   TEXT/event-stream    ; charset=utf-8"
        )));
    }

    #[test]
    fn is_event_stream_reject_invalid_values() {
        assert!(!is_event_stream(&HeaderValue::from_static("plain/text")));
        assert!(!is_event_stream(&HeaderValue::from_static(
            "text/event-but-not-realy"
        )));
    }
}
