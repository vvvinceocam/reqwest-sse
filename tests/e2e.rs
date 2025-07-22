use std::{pin::Pin, time::Duration};

use httpmock::MockServer;

use reqwest_sse::{Event, EventSource, error::EventError};
use tokio_stream::{Stream, StreamExt};

async fn assert_events(
    stream: &mut Pin<Box<impl Stream<Item = Result<Event, EventError>>>>,
    expected_events: &[Event],
) {
    for expected in expected_events {
        let event = stream.next().await.unwrap().unwrap();
        assert_eq!(&event, expected);
    }
}

#[tokio::test]
async fn process_simple_event_stream() {
    let server = MockServer::start_async().await;

    let mock = server
        .mock_async(|when, then| {
            when.method("GET").path("/sse");
            then.status(200)
                .header("content-type", "text/event-stream")
                .body(include_str!("data/simple_event_stream.sse"));
        })
        .await;

    let mut events = reqwest::get(server.url("/sse"))
        .await
        .unwrap()
        .events()
        .await
        .unwrap();

    mock.assert_async().await;

    assert_events(
        &mut events,
        &[
            Event {
                event_type: "message".to_string(),
                data: "first event".to_string(),
                last_event_id: None,
                retry: None,
            },
            Event {
                event_type: "message".to_string(),
                data: "second\nevent\nis\nmultiline".to_string(),
                last_event_id: None,
                retry: None,
            },
            Event {
                event_type: "metadata".to_string(),
                data: "event with custom event type".to_string(),
                last_event_id: None,
                retry: None,
            },
            Event {
                event_type: "message".to_string(),
                data: "fourth valid event".to_string(),
                last_event_id: Some("empty-event-with-id-and-retry".to_string()),
                retry: Some(Duration::from_millis(12345)),
            },
        ],
    )
    .await;

    assert!(events.next().await.is_none());
}
