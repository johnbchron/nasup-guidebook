use miette::{Context, IntoDiagnostic};

#[derive(Debug)]
pub struct Config {
  pub guide_id: usize,
  pub api_key: String,
  pub presenter_custom_list_id: usize,
  pub spreadsheet_id_sessions: String,
  pub spreadsheet_id_presenter_institutions: String,
  pub spreadsheet_id_strands: String,
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

    let presenter_custom_list_id = std::env::var("PRESENTER_CUSTOM_LIST_ID")
      .into_diagnostic()
      .context("missing `PRESENTER_CUSTOM_LIST_ID` env var")?;
    let presenter_custom_list_id = presenter_custom_list_id
      .parse::<usize>()
      .into_diagnostic()
      .context("failed to parse presenter custom list ID")?;

    let spreadsheet_id_sessions = std::env::var("SPREADSHEET_ID_SESSIONS")
      .into_diagnostic()
      .context("missing `SPREADSHEET_ID_SESSIONS` env var")?;
    let spreadsheet_id_presenter_institutions =
      std::env::var("SPREADSHEET_ID_PRESENTER_INSTITUTIONS")
        .into_diagnostic()
        .context("missing `SPREADSHEET_ID_PRESENTER_INSTITUTIONS` env var")?;
    let spreadsheet_id_strands = std::env::var("SPREADSHEET_ID_STRANDS")
      .into_diagnostic()
      .context("missing `SPREADSHEET_ID_STRANDS` env var")?;

    Ok(Self {
      guide_id,
      api_key,
      presenter_custom_list_id,
      spreadsheet_id_sessions,
      spreadsheet_id_presenter_institutions,
      spreadsheet_id_strands,
    })
  }
}
