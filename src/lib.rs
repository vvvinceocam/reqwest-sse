use std::pin::Pin;

use async_stream::try_stream;
use reqwest::{
    Response, StatusCode,
    header::{CONTENT_TYPE, HeaderValue},
};
use tokio::io::AsyncBufReadExt;
use tokio_stream::{Stream, StreamExt};
use tokio_util::io::StreamReader;

pub static MIME_EVENT_STREAM: HeaderValue = HeaderValue::from_static("text/event-stream");

struct EventBuffer {
    event_type: String,
    data: String,
}

impl EventBuffer {
    #[allow(clippy::new_without_default)]
    fn new() -> Self {
        Self {
            event_type: String::new(),
            data: String::new(),
        }
    }

    fn produce_event(&mut self) -> Option<Event> {
        let event = if !self.data.is_empty() {
            Some(Event {
                event_type: if self.event_type.is_empty() {
                    "message".to_string()
                } else {
                    self.event_type.clone()
                },
                data: self.data.to_string(),
            })
        } else {
            None
        };

        self.event_type.clear();
        self.data.clear();

        event
    }

    fn set_event_type(&mut self, event_type: &str) {
        self.event_type.clear();
        self.event_type.push_str(event_type);
    }

    fn push_data(&mut self, data: &str) {
        if !self.data.is_empty() {
            self.data.push('\n');
        }
        self.data.push_str(data);
    }
}

fn parse_line(line: &str) -> (&str, &str) {
    let (field, value) = line.split_once(':').unwrap_or((line, ""));
    let value = value.strip_prefix(' ').unwrap_or(value);
    (field, value)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Event {
    pub event_type: String,
    pub data: String,
}

#[derive(Debug)]
pub enum EventError {
    IoError(std::io::Error),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ServerSentEventsError {
    BadStatus(String),
    BadContentType,
}

pub trait ServerSentEvents {
    fn events(
        self,
    ) -> impl Future<
        Output = Result<
            Pin<Box<impl Stream<Item = Result<Event, EventError>>>>,
            ServerSentEventsError,
        >,
    > + Send;
}

impl ServerSentEvents for Response {
    async fn events(
        self,
    ) -> Result<Pin<Box<impl Stream<Item = Result<Event, EventError>>>>, ServerSentEventsError>
    {
        let status = self.status();
        if status != StatusCode::OK {
            return Err(ServerSentEventsError::BadStatus(format!(
                "expects 200 OK, found {}",
                status.as_str()
            )));
        }
        let content_type = self.headers().get(CONTENT_TYPE);
        if content_type != Some(&MIME_EVENT_STREAM) {
            return Err(ServerSentEventsError::BadContentType);
        }

        let mut stream = StreamReader::new(
            self.bytes_stream()
                .map(|result| result.map_err(std::io::Error::other)),
        );

        let mut line_buffer = String::new();
        let mut event_buffer = EventBuffer::new();

        let stream = Box::pin(try_stream! {
            loop {
                line_buffer.clear();
                let count = stream.read_line(&mut line_buffer).await.map_err(EventError::IoError)?;
                if count == 0 {
                    break;
                }
                let line = if let Some(line) = line_buffer.strip_suffix('\n') {
                    line
                } else {
                    &line_buffer
                };

                // dispatch
                if line.is_empty() {
                    if let Some(event) = event_buffer.produce_event() {
                        yield event;
                    }
                    continue;
                }

                let (field, value) = parse_line(line);

                match field {
                    "event" => {
                        event_buffer.set_event_type(value);
                    }
                    "data" => {
                        event_buffer.push_data(value);
                    }
                    _ => continue,
                }
            }
        });

        Ok(stream)
    }
}

pub async fn assert_events(
    stream: &mut Pin<Box<impl Stream<Item = Result<Event, EventError>>>>,
    expected_events: &[Event],
) {
    for expected in expected_events {
        let event = stream.next().await.unwrap().unwrap();
        assert_eq!(&event, expected);
    }
}
