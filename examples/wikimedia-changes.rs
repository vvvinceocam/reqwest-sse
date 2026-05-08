use reqwest::ClientBuilder;
use tokio_stream::StreamExt;

use reqwest_sse::EventSource;

#[tokio::main]
async fn main() {
    let client = ClientBuilder::new()
        .user_agent("reqwest-sse") // Wikimedia server expects a user agent
        .build()
        .unwrap();

    let mut events = client
        .get("https://stream.wikimedia.org/v2/stream/recentchange")
        .send()
        .await
        .unwrap()
        .events()
        .await
        .unwrap();

    while let Some(Ok(event)) = events.next().await {
        if event.event_type == "message" {
            println!("{} ", event.data);
        }
    }
    println!();
}
