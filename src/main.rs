use futures_util::future;
use futures_util::pin_mut;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::io;
use std::io::Write;

use crossterm::cursor;
use crossterm::execute;
use crossterm::ExecutableCommand;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::prelude::*;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error as WebSocketError;
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use url::Url;

#[derive(Debug)]
struct CursorPosition {
  row: u16,
  col: u16,
  relative: u16,
}

#[derive(Serialize, Deserialize, Debug)]
struct TickerPayload {
  s: String,
  c: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamPayload {
  stream: String,
  data: TickerPayload,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let url = "wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/adausdt@ticker";

  println!("Connecting to {} ...", url);

  let url = Url::parse(url).unwrap();

  println!("Opening websocket...");

  let (ws_stream, res) = connect_async(url).await?;

  println!("Connected to {:?}", res);

  let (_, ws_stream_reader) = ws_stream.split();

  let mut cursor_position_by_stream: HashMap<&str, CursorPosition> = HashMap::new();

  let (_, y) = cursor::position()?;

  cursor_position_by_stream.insert(
    "btcusdt@ticker",
    CursorPosition {
      row: y - 1,
      col: 0,
      relative: 2,
    },
  );
  cursor_position_by_stream.insert(
    "adausdt@ticker",
    CursorPosition {
      row: y - 2,
      col: 0,
      relative: 1,
    },
  );

  println!("----------------------");
  println!();
  println!();

  ws_stream_reader
    .for_each(|message: Result<WebSocketMessage, WebSocketError>| {
      async {
        match message {
          Ok(message) => {
            let body: StreamPayload = serde_json::from_str(message.to_text().unwrap()).unwrap();
            let cursor_position = cursor_position_by_stream.get(&body.stream[..]).unwrap();
            let stdout = io::stdout();

            execute!(io::stdout(), cursor::MoveUp(cursor_position.relative));
            print!("\r{}: {}\r", body.data.s, body.data.c);
            io::stdout().flush().unwrap();
            execute!(io::stdout(), cursor::MoveDown(cursor_position.relative));
          }
          Err(err) => println!("Error: {}", err),
        }
      }
    })
    .await;

  return Ok(());
}
