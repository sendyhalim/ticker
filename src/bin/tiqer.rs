use std::collections::HashMap;
use std::io;
use std::io::Write;

use clap::App as Cli;
use clap::Arg;
use clap::ArgMatches;
use clap::SubCommand;
use crossterm::cursor;
use crossterm::execute;
use env_logger;
use serde::{Deserialize, Serialize};

use lib::binance::ticker;

#[derive(Debug)]
struct CursorPosition {
  symbol_pair: String,
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

fn binance_cmd<'a, 'b>() -> Cli<'a, 'b> {
  let symbol_pairs_arg = Arg::with_name("symbol_pairs")
    .takes_value(true)
    .required(true)
    .min_values(1)
    .help("symbol pairs e.g. 'btcusdt xmrusdt'");

  return SubCommand::with_name("binance")
    .setting(clap::AppSettings::ArgRequiredElseHelp)
    .about("Binance cli")
    .subcommand(
      SubCommand::with_name("ticker")
        .about("Binance ticker")
        .arg(symbol_pairs_arg),
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  env_logger::init();

  let cli = Cli::new("tiqer").subcommand(binance_cmd()).get_matches();

  if let Some(binance_cli) = cli.subcommand_matches("binance") {
    handle_binance_cli(binance_cli).await?;
  }

  return Ok(());
}

async fn handle_binance_cli(cli: &ArgMatches<'_>) -> Result<(), Box<dyn std::error::Error>> {
  if let Some(binance_ticker_cli) = cli.subcommand_matches("ticker") {
    let symbol_pairs: Vec<&str> = binance_ticker_cli
      .values_of("symbol_pairs")
      .unwrap()
      .collect();

    let symbol_pairs: Vec<String> = symbol_pairs
      .iter()
      .map(|symbol_pair| symbol_pair.to_lowercase())
      .collect();

    let ticker_range = std::ops::Range {
      start: 0,
      end: symbol_pairs.len(),
    };

    let ticker_cursors: Vec<(String, CursorPosition)> = ticker_range
      .map(|index| {
        // They will return upper case as the symbol pair
        let symbol_pair = String::from(symbol_pairs.get(index).unwrap()).to_uppercase();

        let cursor_position = CursorPosition {
          symbol_pair: symbol_pair.clone(),
          relative: (symbol_pairs.len() - index) as u16,
        };

        return (symbol_pair, cursor_position);
      })
      .collect();

    let cursor_position_by_symbol_pair: HashMap<String, CursorPosition> =
      ticker_cursors.into_iter().collect();

    println!("----------------------");
    print_lines(&symbol_pairs);

    ticker::start(
      symbol_pairs,
      |body: StreamPayload| {
        let symbol_pair = body.data.s;
        let cursor_position = cursor_position_by_symbol_pair.get(&symbol_pair).unwrap();

        execute!(io::stdout(), cursor::MoveUp(cursor_position.relative)).unwrap();

        print!("\r{}: {}\r", symbol_pair, body.data.c);
        io::stdout().flush().unwrap();

        execute!(io::stdout(), cursor::MoveDown(cursor_position.relative)).unwrap();
      },
      |error| {
        println!("{}", error);
      },
    )
    .await?;
  }

  return Ok(());
}

fn print_lines(tickers: &Vec<String>) {
  println!(
    "{}",
    tickers.iter().map(|_| "\n").collect::<Vec<&str>>().join("")
  );
}
