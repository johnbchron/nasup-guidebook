use miette::Context;

use self::parse_model::{
  ParsedNasupPresenterWithInstitutionBySession, ParsedNasupSession,
};
use crate::{config::Config, fetch_sheet::fetch_xlsx_from_google_sheets};

pub mod parse_model;
pub mod parse_presenter_institutions;
pub mod parse_sessions;

pub async fn fetch_and_parse_nasup_sessions(
  config: &Config,
) -> miette::Result<Vec<ParsedNasupSession>> {
  let mut sessions_sheet =
    fetch_xlsx_from_google_sheets(&config.spreadsheet_id_sessions).await?;
  let sessions_worksheet = sessions_sheet
    .get_worksheet("2026 Detailed Schedule")
    .context("failed to get correct worksheet from sessions sheet")?;

  self::parse_sessions::parse_nasup_sessions_from_worksheet(sessions_worksheet)
    .context("failed to parse nasup session data from spreadsheet")
}

pub async fn fetch_and_parse_nasup_presenter_institutions(
  config: &Config,
) -> miette::Result<Vec<ParsedNasupPresenterWithInstitutionBySession>> {
  let mut presenter_institutions_sheet = fetch_xlsx_from_google_sheets(
    &config.spreadsheet_id_presenter_institutions,
  )
  .await?;
  let presenter_institutions_worksheet = presenter_institutions_sheet
    .get_worksheet("oa_export.xlsx")
    .context(
      "failed to get correct worksheet from presenter institutions sheet",
    )?;
  self::parse_presenter_institutions::parse_nasup_presenter_institutions_from_worksheet(presenter_institutions_worksheet).context("failed to parse nasup presenter institution data from spreadsheet")
}
