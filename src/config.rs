use miette::{Context, IntoDiagnostic};

#[derive(Debug)]
pub struct Config {
  guide_id: usize,
  api_key:  String,
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

    Ok(Self { guide_id, api_key })
  }
}
