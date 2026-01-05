#![feature(iterator_try_collect)]
#![feature(int_roundings)]

mod config;

mod fetch_sheet;
mod parse_nasup;

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

  // let mut sessions_sheet =
  //   fetch_xlsx_from_google_sheets(&config.spreadsheet_id_sessions).await?;
  // let sessions_worksheet = sessions_sheet
  //   .get_worksheet("2026 Detailed Schedule")
  //   .context("failed to get correct worksheet from sessions sheet")?;

  // let _parsed_nasup_session_data =
  //   self::parse_nasup::parse_sessions::parse_nasup_sessions_from_worksheet(
  //     sessions_worksheet,
  //   )
  //   .context("failed to parse nasup session data from spreadsheet")?;

  let mut presenter_institutions_sheet = fetch_xlsx_from_google_sheets(
    &config.spreadsheet_id_presenter_institutions,
  )
  .await?;
  let presenter_institutions_worksheet = presenter_institutions_sheet
    .get_worksheet("oa_export.xlsx")
    .context(
      "failed to get correct worksheet from presenter institutions sheet",
    )?;
  let _parsed_nasup_presenter_institutions_data = self::parse_nasup::parse_presenter_institutions::parse_nasup_presenter_institutions_from_worksheet(presenter_institutions_worksheet).context("failed to parse nasup presenter institution data from spreadsheet")?;

  // let session_json = serde_json::to_string(&parsed_nasup_session_data)
  //   .into_diagnostic()
  //   .context("failed to serialize parsed nasup session data")?;

  // std::fs::write("/tmp/nasup_data.json", &session_json)
  //   .into_diagnostic()
  //   .context("failed to write parsed nasup data as JSON")?;

  Ok(())
}
