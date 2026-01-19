use kinded::Kinded;
use miette::Context;
use tracing::info;

use crate::{
  config::Config,
  fetch_sheet::{DecodedWorksheet, fetch_xlsx_from_google_sheets},
  guidebook::{fetch_all_guidebook_sessions, model::GuidebookSession},
  nasup_to_guidebook::nasup_session_to_guidebook_session,
  parse_nasup::{
    parse_model::{
      ParsedNasupPresenterWithInstitutionBySession, ParsedNasupSession,
      ParsedNasupStrandsAndIntendedAudience,
    },
    parse_presenter_institutions::parse_nasup_presenter_institutions_from_worksheet,
    parse_sessions::parse_nasup_sessions_from_worksheet,
    parse_strands::parse_nasup_strands_from_worksheet,
  },
  reconcile_guidebook_sessions::{
    SessionReconciliation, reconcile_intended_and_existing_guidebook_sessions,
  },
  synth_nasup::{NasupSession, synthesize_parsed_nasup_data},
};

#[derive(Kinded)]
#[kinded(kind = MasterStateStep, derive(Debug))]
pub enum MasterState {
  Start,
  FetchedSheets {
    sessions_worksheet:  DecodedWorksheet,
    presenter_worksheet: DecodedWorksheet,
    strands_worksheet:   DecodedWorksheet,
  },
  ParsedInputs {
    sessions:   Vec<ParsedNasupSession>,
    presenters: Vec<ParsedNasupPresenterWithInstitutionBySession>,
    strands:    Vec<ParsedNasupStrandsAndIntendedAudience>,
  },
  SynthesizedInputs {
    sessions: Vec<NasupSession>,
  },
  FetchedGuidebookState {
    intended_sessions: Vec<GuidebookSession>,
    existing_sessions: Vec<GuidebookSession>,
  },
  CalculatedReconciliation {
    session_reconciliation: SessionReconciliation,
  },
  ExecutedReconciliation,
}

impl MasterState {
  pub fn completed(&self) -> bool {
    matches!(self, Self::ExecutedReconciliation)
  }

  pub async fn step(self, config: &Config) -> miette::Result<Self> {
    let old_state_step = self.kind();
    let new_state: MasterState = match self {
      MasterState::Start => MasterState::FetchedSheets {
        sessions_worksheet:  fetch_xlsx_from_google_sheets(
          &config.spreadsheet_id_sessions,
        )
        .await?
        .get_worksheet("2026 Detailed Schedule")
        .context("failed to get correct worksheet from sessions sheet")?,
        presenter_worksheet: fetch_xlsx_from_google_sheets(
          &config.spreadsheet_id_presenter_institutions,
        )
        .await?
        .get_worksheet("oa_export.xlsx")
        .context(
          "failed to get correct worksheet from presenter institutions sheet",
        )?,
        strands_worksheet:   fetch_xlsx_from_google_sheets(
          &config.spreadsheet_id_strands,
        )
        .await?
        .get_worksheet("oa_export.xlsx")
        .context("failed to get correct worksheet from strands spreadsheet")?,
      },
      MasterState::FetchedSheets {
        sessions_worksheet,
        presenter_worksheet,
        strands_worksheet,
      } => MasterState::ParsedInputs {
        sessions:   parse_nasup_sessions_from_worksheet(sessions_worksheet)
          .context("failed to parse nasup session data from spreadsheet")?,
        presenters: parse_nasup_presenter_institutions_from_worksheet(
          presenter_worksheet,
        )
        .context(
          "failed to parse nasup presenter institution data from spreadsheet",
        )?,
        strands:    parse_nasup_strands_from_worksheet(strands_worksheet)
          .context("failed to parse nasup strands data from spreadsheet")?,
      },
      MasterState::ParsedInputs {
        sessions,
        presenters,
        strands,
      } => MasterState::SynthesizedInputs {
        sessions: synthesize_parsed_nasup_data(sessions, presenters, strands)
          .context("failed to synthesize nasup data")?,
      },
      MasterState::SynthesizedInputs { sessions } => {
        MasterState::FetchedGuidebookState {
          intended_sessions: sessions
            .into_iter()
            .map(|ns| {
              nasup_session_to_guidebook_session(config, ns)
                .context("failed to convert nasup session to guidebook session")
            })
            .try_collect::<Vec<_>>()?,
          existing_sessions: fetch_all_guidebook_sessions(config).await?,
        }
      }
      MasterState::FetchedGuidebookState {
        intended_sessions,
        existing_sessions,
      } => MasterState::CalculatedReconciliation {
        session_reconciliation:
          reconcile_intended_and_existing_guidebook_sessions(
            &intended_sessions,
            &existing_sessions,
          )
          .context(
            "failed to reconcile intended and existing guidebook sessions",
          )?,
      },
      MasterState::CalculatedReconciliation {
        session_reconciliation,
      } => {
        // session_reconciliation
        //   .execute_reconciliation(config)
        //   .await
        //   .context(
        //     "failed to reconcile intended and existing guidebook sessions",
        //   )?;
        // warn!("did not execute reconciliation: dry run");

        MasterState::ExecutedReconciliation
      }
      MasterState::ExecutedReconciliation => unreachable!(),
    };

    info!(
      old_state = ?old_state_step,
      new_state = ?(new_state.kind()),
      "successfully transitioned state"
    );
    Ok(new_state)
  }
}
