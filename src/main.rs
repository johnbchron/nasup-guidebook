#![feature(iterator_try_collect)]
#![feature(int_roundings)]
#![feature(pattern)]

mod config;
mod fetch_guidebook;
mod fetch_sheet;
mod parse_nasup;
mod synth_nasup;

use std::sync::LazyLock;

use miette::Context;
use tracing::warn;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use self::{config::Config, fetch_guidebook::fetch_all_guidebook_sessions};

static HTTP_CLIENT: LazyLock<reqwest::Client> =
  LazyLock::new(reqwest::Client::new);

#[tokio::main]
async fn main() -> miette::Result<()> {
  tracing_subscriber::registry()
    .with(fmt::layer())
    .with(EnvFilter::from_default_env())
    .init();

  let config =
    Config::from_env().context("failed to gather config from env")?;

  // let parsed_nasup_session_data =
  //   self::parse_nasup::fetch_and_parse_nasup_sessions(&config).await?;

  // let parsed_nasup_presenter_institutions_data =
  //   self::parse_nasup::fetch_and_parse_nasup_presenter_institutions(&config)
  //     .await?;

  // let synthesized_sessions = self::synth_nasup::synthesize_parsed_nasup_data(
  //   parsed_nasup_session_data,
  //   parsed_nasup_presenter_institutions_data,
  // )
  // .context("failed to synthesize nasup data")?;

  // let result_json = serde_json::to_string(&synthesized_sessions)
  //   .into_diagnostic()
  //   .context("failed to serialize parsed nasup session data")?;

  // std::fs::write("/tmp/nasup_data.json", &result_json)
  //   .into_diagnostic()
  //   .context("failed to write parsed nasup data as JSON")?;

  let guidebook_sessions = fetch_all_guidebook_sessions(&config).await?;
  warn!(
    "guidebook session serialized: {}",
    serde_json::to_string_pretty(&guidebook_sessions).unwrap()
  );

  Ok(())
}
