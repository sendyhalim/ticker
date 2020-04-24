use futures_util::future;
use futures_util::pin_mut;
use futures_util::StreamExt;
use std::io;
use std::io::Write;

use serde_json::Value as JsonValue;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::prelude::*;
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error as WebSocketError;
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use url::Url;

struct Counter {
  val: i32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let url = "wss://stream.binance.com:9443/ws/btcusdt@ticker";

  println!("Connecting to {} ...", url);

  let url = Url::parse(url).unwrap();

  println!("Opening websocket...");

  let (ws_stream, res) = connect_async(url).await?;

  println!("Connected to {:?}", res);

  let (_, ws_stream_reader) = ws_stream.split();

  ws_stream_reader
    .for_each(|message: Result<WebSocketMessage, WebSocketError>| {
      async {
        match message {
          Ok(message) => {
            let body: JsonValue = serde_json::from_str(message.to_text().unwrap()).unwrap();

            print!("Ticker: {}\r", body["c"]);

            io::stdout().flush().unwrap();
          }
          Err(err) => println!("Error: {}", err),
        }
      }
    })
    .await;

  return Ok(());
}
