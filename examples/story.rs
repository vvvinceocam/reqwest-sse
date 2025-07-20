use std::io::Write;

use tokio_stream::StreamExt;

use reqwest_sse::ServerSentEvents;

#[tokio::main]
async fn main() {
    let mut events = reqwest::get("https://sse.test-free.online/api/story")
        .await
        .unwrap()
        .events()
        .await
        .unwrap();

    while let Some(Ok(event)) = events.next().await {
        if event.event_type == "message" {
            print!("{} ", event.data);
            std::io::stdout().flush().unwrap();
        }
    }
    println!();
}
