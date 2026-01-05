use chrono::{Datelike, TimeZone, Timelike};
use chrono_tz::{Tz, US::Eastern};
use serde::Serialize;
use tracing::{debug, error};

use crate::parse_nasup::parse_model::{
  ParsedNasupPresenterWithInstitutionBySession, ParsedNasupSession,
  ParsedNasupSessionType,
};

#[derive(Clone, Debug, Serialize)]
pub struct NasupSession {
  pub start_datetime:      chrono::DateTime<Tz>,
  pub end_datetime:        chrono::DateTime<Tz>,
  pub room:                String,
  pub session_type:        ParsedNasupSessionType,
  pub title:               String,
  pub description:         String,
  /// Only presenters that have paid are included here.
  pub approved_presenters: Vec<NasupPresenter>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NasupPresenter {
  pub name:               String,
  pub first_institution:  Option<String>,
  pub second_institution: Option<String>,
}

pub fn synthesize_parsed_nasup_data(
  parsed_sessions: Vec<ParsedNasupSession>,
  parsed_presenter_institutions: Vec<
    ParsedNasupPresenterWithInstitutionBySession,
  >,
) -> miette::Result<Vec<NasupSession>> {
  let mut synthesized_sessions = Vec::new();

  for parsed_session in parsed_sessions {
    let start_datetime = Eastern
      .with_ymd_and_hms(
        parsed_session.date.year(),
        parsed_session.date.month(),
        parsed_session.date.day(),
        parsed_session.start_time.hour(),
        parsed_session.start_time.minute(),
        parsed_session.start_time.second(),
      )
      .single()
      .expect("super crazy time weirdness");
    let end_datetime = Eastern
      .with_ymd_and_hms(
        parsed_session.date.year(),
        parsed_session.date.month(),
        parsed_session.date.day(),
        parsed_session.end_time.hour(),
        parsed_session.end_time.minute(),
        parsed_session.end_time.second(),
      )
      .single()
      .expect("super crazy time weirdness");

    let relevant_presenter_institution_records = parsed_presenter_institutions
      .iter()
      .filter(|r| r.session_name == parsed_session.title)
      .cloned()
      .collect::<Vec<_>>();

    let mut approved_presenters = Vec::new();

    let paid_presenter_iter =
      parsed_session.presenters.iter().filter(|p| p.paid);
    for paid_presenter in paid_presenter_iter {
      let record = relevant_presenter_institution_records
        .iter()
        .find(|r| r.name == paid_presenter.name);

      match record {
        Some(record) => {
          approved_presenters.push(NasupPresenter {
            name:               paid_presenter.name.clone(),
            first_institution:  record.first_institution.clone(),
            second_institution: record.second_institution.clone(),
          });
        }
        None => {
          error!(
            name = paid_presenter.name,
            session = parsed_session.title,
            "could not find presenter-institution-session for presenter"
          );
          approved_presenters.push(NasupPresenter {
            name:               paid_presenter.name.clone(),
            first_institution:  None,
            second_institution: None,
          });
        }
      }
    }

    let synthesized_session = NasupSession {
      start_datetime,
      end_datetime,
      room: parsed_session.room,
      session_type: parsed_session.session_type,
      title: parsed_session.title,
      description: parsed_session.description,
      approved_presenters,
    };

    debug!("synthesized full session record: {synthesized_session:#?}");

    synthesized_sessions.push(synthesized_session);
  }

  Ok(synthesized_sessions)
}
