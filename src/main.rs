#![feature(iterator_try_collect)]
#![feature(int_roundings)]
#![feature(pattern)]

mod config;
mod fetch_sheet;
mod guidebook;
mod nasup_to_guidebook;
mod parse_nasup;
mod reconcile_guidebook;
mod synth_nasup;

use std::sync::LazyLock;

use miette::Context;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use self::{
  config::Config, guidebook::fetch_all_guidebook_sessions,
  nasup_to_guidebook::nasup_session_to_guidebook_session,
  reconcile_guidebook::reconcile_intended_and_existing_guidebook_sessions,
};

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

  let parsed_nasup_session_data =
    self::parse_nasup::fetch_and_parse_nasup_sessions(&config).await?;

  let parsed_nasup_presenter_institutions_data =
    self::parse_nasup::fetch_and_parse_nasup_presenter_institutions(&config)
      .await?;

  let synthesized_sessions = self::synth_nasup::synthesize_parsed_nasup_data(
    parsed_nasup_session_data,
    parsed_nasup_presenter_institutions_data,
  )
  .context("failed to synthesize nasup data")?;

  let intended_guidebook_sessions = synthesized_sessions
    .into_iter()
    .map(|ns| {
      nasup_session_to_guidebook_session(&config, ns)
        .context("failed to convert nasup session to guidebook session")
    })
    .try_collect::<Vec<_>>()?;

  let existing_guidebook_sessions =
    fetch_all_guidebook_sessions(&config).await?;

  let session_reconciliation =
    reconcile_intended_and_existing_guidebook_sessions(
      &intended_guidebook_sessions,
      &existing_guidebook_sessions,
    )
    .context("failed to reconcile intended and existing guidebook sessions")?;

  dbg!(&session_reconciliation);

  // session_reconciliation
  //   .execute_reconciliation(&config)
  //   .await
  //   .context("failed to execute reconciliation")?;

  Ok(())
}
