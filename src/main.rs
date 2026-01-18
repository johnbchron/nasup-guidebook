#![feature(iterator_try_collect)]
#![feature(int_roundings)]
#![feature(pattern)]
#![feature(iter_intersperse)]

mod config;
mod fetch_sheet;
mod guidebook;
mod nasup_to_guidebook;
mod parse_nasup;
mod reconcile_guidebook_sessions;
mod state;
mod synth_nasup;

use std::sync::LazyLock;

use miette::Context;
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use self::{config::Config, state::MasterState};

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

  // drive state machine
  let mut state = MasterState::Start;
  loop {
    match state {
      s if s.completed() => {
        info!("state machine completed");
        break;
      }
      s => {
        state = s.step(&config).await.context("failed to step state")?;
      }
    }
  }

  Ok(())
}
