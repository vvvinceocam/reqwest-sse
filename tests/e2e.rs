use httpmock::MockServer;

use reqwest_sse::{Event, ServerSentEvents, assert_events};
use tokio_stream::StreamExt;

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
            },
            Event {
                event_type: "message".to_string(),
                data: "second\nevent\nis\nmultiline".to_string(),
            },
            Event {
                event_type: "metadata".to_string(),
                data: "event with custom event type".to_string(),
            },
            Event {
                event_type: "message".to_string(),
                data: "fourth valid event".to_string(),
            },
        ],
    )
    .await;

    assert!(events.next().await.is_none());
}
