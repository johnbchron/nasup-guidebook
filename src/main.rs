#![feature(iterator_try_collect)]

mod config;

mod fetch_sheet;
mod nasup;

use std::sync::LazyLock;

use miette::Context;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use self::{config::Config, fetch_sheet::fetch_xlsx_from_google_sheets};

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

  let mut sheet =
    fetch_xlsx_from_google_sheets(&config.spreadsheet_id_room_setup).await?;
  let worksheet = sheet.get_worksheet("2026 Detailed Schedule").context(
    "failed to get detailed schedule worksheet from room setup sheet",
  )?;
  // let worksheet_data = worksheet.range(
  //   (
  //     worksheet.start().unwrap().0 + 1,
  //     worksheet.start().unwrap().1,
  //   ),
  //   worksheet.end().unwrap(),
  // );

  let nasup_data =
    self::nasup::parse_nasup_sessions_from_xlsx_range(worksheet)?;

  Ok(())
}
