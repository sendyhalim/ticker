use futures_util::StreamExt;
use std::collections::HashMap;
use std::io;
use std::io::Write;

use crossterm::cursor;
use crossterm::execute;
use serde::{Deserialize, Serialize};
use serde_json::Result as SerdeResult;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error as WebSocketError;
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use url::Url;

#[derive(Debug)]
struct CursorPosition {
  ticker_symbol: String,
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
  let tickers: Vec<String> = vec!["btcusdt", "adausdt", "iotausdt", "xlmusdt"]
    .iter()
    .map(|symbol_pair| {
      return format!("{}@ticker", symbol_pair);
    })
    .collect();

  let stream_params: String = tickers.join("/");

  let url = format!(
    "wss://stream.binance.com:9443/stream?streams={}",
    stream_params
  );

  let ticker_range = std::ops::Range {
    start: 0,
    end: tickers.len(),
  };

  let ticker_cursors: Vec<(String, CursorPosition)> = ticker_range
    .map(|index| {
      let ticker_symbol = tickers.get(index).unwrap();

      let cursor_position = CursorPosition {
        ticker_symbol: String::from(ticker_symbol),
        relative: (tickers.len() - index) as u16,
      };

      return (String::from(ticker_symbol), cursor_position);
    })
    .collect();

  let cursor_position_by_stream: HashMap<String, CursorPosition> =
    ticker_cursors.into_iter().collect();

  println!("Connecting to {} ...", url);

  let url = Url::parse(&url).unwrap();

  println!("Opening websocket...");

  let (ws_stream, res) = connect_async(url).await?;

  println!("Connected to {:?}", res);

  let (_, ws_stream_reader) = ws_stream.split();

  println!("{:?}", cursor_position_by_stream);

  println!("----------------------");
  print_lines(&tickers);

  ws_stream_reader
    .for_each(|message: Result<WebSocketMessage, WebSocketError>| {
      async {
        match message {
          Ok(message) => {
            let response_text = message.to_text().unwrap();
            let body: SerdeResult<StreamPayload> = serde_json::from_str(response_text);

            if body.is_err() {
              println!("Error when deserializing {}", response_text);
              print_lines(&tickers);
              return;
            }

            let body = body.unwrap();

            let cursor_position = cursor_position_by_stream.get(&body.stream[..]).unwrap();

            execute!(io::stdout(), cursor::MoveUp(cursor_position.relative)).unwrap();

            print!("\r{}: {}\r", body.data.s, body.data.c);
            io::stdout().flush().unwrap();

            execute!(io::stdout(), cursor::MoveDown(cursor_position.relative)).unwrap();
          }
          Err(err) => println!("Error: {}", err),
        }
      }
    })
    .await;

  return Ok(());
}

fn print_lines(tickers: &Vec<String>) {
  println!(
    "{}",
    tickers.iter().map(|_| "\n").collect::<Vec<&str>>().join("")
  );
}
