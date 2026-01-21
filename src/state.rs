use std::collections::HashMap;

use kinded::Kinded;
use miette::Context;
use tracing::info;

use crate::{
  config::Config,
  fetch_sheet::{DecodedWorksheet, fetch_xlsx_from_google_sheets},
  guidebook::{
    fetch_all_guidebook_entities,
    model::{GuidebookPresenter, GuidebookScheduleTrack, GuidebookSession},
  },
  nasup_to_guidebook::{
    WithLinks, nasup_session_to_guidebook_session,
    nasup_sessions_to_guidebook_presenters,
    nasup_sessions_to_guidebook_schedule_tracks,
  },
  parse_nasup::{
    parse_model::{
      ParsedNasupPresenterWithInstitutionBySession, ParsedNasupSession,
      ParsedNasupStrandAndIntendedAudience,
    },
    parse_presenter_institutions::parse_nasup_presenter_institutions_from_worksheet,
    parse_sessions::parse_nasup_sessions_from_worksheet,
    parse_strands::parse_nasup_strands_from_worksheet,
  },
  reconcile_guidebook_presenters::{
    PresenterReconciliation,
    reconcile_intended_and_existing_guidebook_presenters,
  },
  reconcile_guidebook_sessions::{
    SessionReconciliation, reconcile_intended_and_existing_guidebook_sessions,
  },
  reconcile_guidebook_strands::{
    StrandsReconciliation,
    reconcile_intended_and_existing_guidebook_schedule_tracks,
  },
  synchronize_links::synchronize_session_links,
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
    strands:    Vec<ParsedNasupStrandAndIntendedAudience>,
  },
  SynthesizedInputs {
    sessions: Vec<NasupSession>,
  },
  FetchedStrandsState {
    sessions:         Vec<NasupSession>,
    intended_strands: Vec<GuidebookScheduleTrack>,
    existing_strands: Vec<GuidebookScheduleTrack>,
  },
  CalculatedStrandsReconciliation {
    sessions:               Vec<NasupSession>,
    strands_reconciliation: StrandsReconciliation,
  },
  ExecutedStrandsReconciliation {
    sessions:         Vec<NasupSession>,
    existing_strands: Vec<GuidebookScheduleTrack>,
  },
  FetchedGuidebookPresenterState {
    sessions:            Vec<NasupSession>,
    existing_strands:    Vec<GuidebookScheduleTrack>,
    intended_presenters: Vec<GuidebookPresenter>,
    existing_presenters: Vec<GuidebookPresenter>,
  },
  CalculatedPresenterReconciliation {
    sessions:                 Vec<NasupSession>,
    existing_strands:         Vec<GuidebookScheduleTrack>,
    presenter_reconciliation: PresenterReconciliation,
  },
  ExecutedPresenterReconciliation {
    sessions:            Vec<NasupSession>,
    existing_strands:    Vec<GuidebookScheduleTrack>,
    existing_presenters: Vec<GuidebookPresenter>,
  },
  FetchedGuidebookSessionState {
    intended_sessions: Vec<GuidebookSession>,
    existing_sessions: Vec<GuidebookSession>,
    intended_session_import_id_to_presenter_link_map: HashMap<String, Vec<u32>>,
  },
  CalculatedSessionReconciliation {
    session_reconciliation: SessionReconciliation,
    intended_session_import_id_to_presenter_link_map: HashMap<String, Vec<u32>>,
  },
  ExecutedSessionReconciliation {
    intended_session_to_presenter_link_map: HashMap<u32, Vec<u32>>,
  },
  SynchronizedLinks,
}

impl MasterState {
  pub fn completed(&self) -> bool { matches!(self, Self::SynchronizedLinks) }

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
        MasterState::FetchedStrandsState {
          sessions:         sessions.clone(),
          intended_strands: nasup_sessions_to_guidebook_schedule_tracks(
            config,
            sessions.as_slice(),
          )?,
          existing_strands: fetch_all_guidebook_entities(
            config,
            "/schedule-tracks",
          )
          .await?,
        }
      }

      MasterState::FetchedStrandsState {
        sessions,
        intended_strands,
        existing_strands,
      } => MasterState::CalculatedStrandsReconciliation {
        sessions,
        strands_reconciliation:
          reconcile_intended_and_existing_guidebook_schedule_tracks(
            &intended_strands,
            &existing_strands,
          )
          .context(
            "failed to reconcile intended and existing guidebook session \
             tracks",
          )?,
      },

      MasterState::CalculatedStrandsReconciliation {
        sessions,
        strands_reconciliation,
      } => {
        strands_reconciliation
          .execute_reconciliation(config)
          .await
          .context(
            "failed to reconcile intended and existing guidebook session \
             tracks",
          )?;

        MasterState::ExecutedStrandsReconciliation {
          sessions,
          existing_strands: fetch_all_guidebook_entities(
            config,
            "/schedule-tracks",
          )
          .await?,
        }
      }

      MasterState::ExecutedStrandsReconciliation {
        sessions,
        existing_strands,
      } => MasterState::FetchedGuidebookPresenterState {
        sessions: sessions.clone(),
        existing_strands,
        intended_presenters: nasup_sessions_to_guidebook_presenters(
          config, &sessions,
        )
        .context("failed to extract nasup presenters from nasup sessions")?,
        existing_presenters: fetch_all_guidebook_entities(
          config,
          &format!(
            "/custom-list-items/?custom_lists={list_id}",
            list_id = config.presenter_custom_list_id
          ),
        )
        .await?,
      },

      MasterState::FetchedGuidebookPresenterState {
        sessions,
        existing_strands,
        intended_presenters,
        existing_presenters,
      } => MasterState::CalculatedPresenterReconciliation {
        sessions,
        existing_strands,
        presenter_reconciliation:
          reconcile_intended_and_existing_guidebook_presenters(
            &intended_presenters,
            &existing_presenters,
          )
          .context(
            "failed to reconcile intended and existing guidebook presenters",
          )?,
      },

      MasterState::CalculatedPresenterReconciliation {
        sessions,
        existing_strands,
        presenter_reconciliation,
      } => {
        presenter_reconciliation
          .execute_reconciliation(config)
          .await
          .context(
            "failed to reconcile intended and existing guidebook presenters",
          )?;
        MasterState::ExecutedPresenterReconciliation {
          sessions,
          existing_strands,
          existing_presenters: fetch_all_guidebook_entities(
            config,
            &format!(
              "/custom-list-items/?custom_lists={list_id}",
              list_id = config.presenter_custom_list_id
            ),
          )
          .await?,
        }
      }

      MasterState::ExecutedPresenterReconciliation {
        sessions,
        existing_strands,
        existing_presenters,
      } => {
        let intended_sessions = sessions
          .into_iter()
          .map(|ns| {
            nasup_session_to_guidebook_session(
              config,
              ns,
              &existing_strands,
              &existing_presenters,
            )
            .context("failed to convert nasup session to guidebook session")
          })
          .try_collect::<Vec<_>>()?;
        let mut import_id_to_links_map = HashMap::new();
        let intended_sessions = intended_sessions
          .into_iter()
          .map(|WithLinks(s, links)| {
            import_id_to_links_map.insert(s.import_id.clone().unwrap(), links);
            s
          })
          .collect();

        MasterState::FetchedGuidebookSessionState {
          intended_sessions,
          existing_sessions: fetch_all_guidebook_entities(config, "/sessions")
            .await?,
          intended_session_import_id_to_presenter_link_map:
            import_id_to_links_map,
        }
      }

      MasterState::FetchedGuidebookSessionState {
        intended_sessions,
        existing_sessions,
        intended_session_import_id_to_presenter_link_map,
      } => MasterState::CalculatedSessionReconciliation {
        session_reconciliation:
          reconcile_intended_and_existing_guidebook_sessions(
            &intended_sessions,
            &existing_sessions,
          )
          .context(
            "failed to reconcile intended and existing guidebook sessions",
          )?,
        intended_session_import_id_to_presenter_link_map,
      },

      MasterState::CalculatedSessionReconciliation {
        session_reconciliation,
        intended_session_import_id_to_presenter_link_map,
      } => {
        session_reconciliation
          .execute_reconciliation(config)
          .await
          .context(
            "failed to reconcile intended and existing guidebook sessions",
          )?;

        let new_session_state =
          fetch_all_guidebook_entities::<GuidebookSession>(config, "/sessions")
            .await?;
        let intended_session_to_presenter_link_map =
          intended_session_import_id_to_presenter_link_map
            .into_iter()
            .filter_map(|(iid, links)| {
              new_session_state
                .iter()
                .find(|s| s.import_id.as_ref().unwrap() == &iid)
                .map(|s| (s.id.unwrap(), links))
            })
            .collect::<HashMap<_, _>>();
        MasterState::ExecutedSessionReconciliation {
          intended_session_to_presenter_link_map,
        }
      }

      MasterState::ExecutedSessionReconciliation {
        intended_session_to_presenter_link_map,
      } => {
        synchronize_session_links(
          config,
          intended_session_to_presenter_link_map,
        )
        .await
        .context("failed to synchronize links")?;
        MasterState::SynchronizedLinks
      }

      MasterState::SynchronizedLinks => unreachable!(),
    };

    info!(
      old_state = ?old_state_step,
      new_state = ?(new_state.kind()),
      "successfully transitioned state"
    );
    Ok(new_state)
  }
}
