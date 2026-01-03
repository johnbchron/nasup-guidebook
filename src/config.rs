use miette::{Context, IntoDiagnostic};

#[derive(Debug)]
pub struct Config {
  pub guide_id:                  usize,
  pub api_key:                   String,
  pub spreadsheet_id_room_setup: String,
}

impl Config {
  pub fn from_env() -> miette::Result<Self> {
    let guide_id = std::env::var("GUIDE_ID")
      .into_diagnostic()
      .context("missing `GUIDE_ID` env var")?;
    let guide_id = guide_id
      .parse::<usize>()
      .into_diagnostic()
      .context("failed to parse guide ID")?;

    let api_key = std::env::var("API_KEY")
      .into_diagnostic()
      .context("missing `API_KEY` env var")?;

    let spreadsheet_id_room_setup = std::env::var("SPREADSHEET_ID_ROOM_SETUP")
      .into_diagnostic()
      .context("missing `SPREADSHEET_ID_ROOM_SETUP` env var")?;

    Ok(Self {
      guide_id,
      api_key,
      spreadsheet_id_room_setup,
    })
  }
}
