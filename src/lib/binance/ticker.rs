use failure::Fail;
use futures_util::StreamExt;
use log;
use serde::de::DeserializeOwned;
use serde_json::Result as SerdeResult;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::error::Error as WebSocketError;
use tokio_tungstenite::tungstenite::protocol::Message as WebSocketMessage;
use url::Url;

#[derive(Debug, Clone, Fail)]
pub enum TickerError {
  #[fail(display = "Error when deserializing message {}", message)]
  ParsingError { message: WebSocketMessage },

  #[fail(display = "Error when receiving message {}", message)]
  SocketError { message: String },
}

pub async fn start<OnMessage, OnError, T>(
  symbol_pairs: Vec<String>,
  on_message: OnMessage,
  on_error: OnError,
) -> Result<(), Box<dyn std::error::Error>>
where
  T: DeserializeOwned, // https://serde.rs/lifetimes.html#trait-bounds
  OnMessage: Fn(T) -> (),
  OnError: Fn(TickerError) -> (),
{
  let symbol_pairs: Vec<String> = symbol_pairs
    .iter()
    .map(|symbol_pair| {
      return format!("{}@ticker", symbol_pair);
    })
    .collect();

  let stream_params: String = symbol_pairs.join("/");

  let url = format!(
    "wss://stream.binance.com:9443/stream?streams={}",
    stream_params
  );

  log::debug!("Connecting to {} ...", url);

  let url = Url::parse(&url).unwrap();

  log::debug!("Opening websocket...");

  let (ws_stream, res) = connect_async(url).await?;

  log::debug!("Connected to {:?}", res);

  let (_, ws_stream_reader) = ws_stream.split();

  ws_stream_reader
    .for_each(|message: Result<WebSocketMessage, WebSocketError>| {
      async {
        log::debug!("Got message {:?}", message);

        if let Err(error) = message {
          on_error(TickerError::SocketError {
            message: format!("{}", error),
          });

          return;
        }

        let message = message.unwrap();
        let response_text = message.to_text();

        if response_text.is_err() {
          on_error(TickerError::ParsingError { message });
          return;
        }

        let body: SerdeResult<T> = serde_json::from_str(response_text.unwrap());

        if body.is_err() {
          on_error(TickerError::ParsingError { message });
          return;
        }

        let body = body.unwrap();
        on_message(body);
      }
    })
    .await;

  return Ok(());
}
