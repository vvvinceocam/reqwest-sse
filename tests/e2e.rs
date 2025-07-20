use httpmock::MockServer;

use reqwest_sse::{Event, ServerSentEvents, assert_events};

#[tokio::test]
async fn plop_plip() {
    let server = MockServer::start_async().await;

    let mock = server
        .mock_async(|when, then| {
            when.method("GET").path("/sse");
            then.status(200)
                .header("content-type", "text/event-stream")
                .body(
                    r#"
data: foo

data: foo
data: bar
data: baz

event: coin
data: prout

event: foo

:

id: plop
retry: 12342

data:

nodata

data: asdsadsadsasadsad

"#,
                );
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
                data: "foo".to_string(),
            },
            Event {
                event_type: "message".to_string(),
                data: "foo\nbar\nbaz".to_string(),
            },
            Event {
                event_type: "coin".to_string(),
                data: "prout".to_string(),
            },
            Event {
                event_type: "message".to_string(),
                data: "asdsadsadsasadsad".to_string(),
            },
        ],
    )
    .await;
}
