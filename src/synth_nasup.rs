use std::str::pattern::Pattern;

use chrono::{Datelike, TimeZone, Timelike, Utc};
use chrono_tz::US::Eastern;
use miette::{Context, IntoDiagnostic};
use serde::Serialize;
use tracing::{debug, warn};

use crate::parse_nasup::parse_model::{
  ParsedNasupPresenterWithInstitutionBySession, ParsedNasupSession,
  ParsedNasupSessionType, ParsedNasupStrandAndIntendedAudience,
};

#[derive(Clone, Debug, Serialize)]
pub struct NasupSession {
  pub start_datetime:      chrono::DateTime<Utc>,
  pub end_datetime:        chrono::DateTime<Utc>,
  pub room:                String,
  pub session_type:        ParsedNasupSessionType,
  pub title:               String,
  pub description:         String,
  /// Only presenters that have paid are included here.
  pub approved_presenters: Vec<NasupPresenter>,
  pub strand:              Option<String>,
  pub intended_audience:   Option<String>,
  pub rank:                f32,
}

impl NasupSession {
  pub fn primary_key(&self) -> String {
    let session_primary_key = serde_json::json!({
      "name": self.title.get(0..30).unwrap_or(&self.title),
      "room": self.room,
      "start": self.start_datetime,
      "end": self.end_datetime,
    });

    serde_json::to_string(&session_primary_key)
      .into_diagnostic()
      .context("failed to serialize session primary key into JSON")
      .unwrap()
  }
}

#[derive(Clone, Debug, Serialize, Hash, PartialEq, Eq)]
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
  parsed_strands: Vec<ParsedNasupStrandAndIntendedAudience>,
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
      .expect("super crazy time weirdness")
      .to_utc();
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
      .expect("super crazy time weirdness")
      .to_utc();

    let session_name_search_query =
      strip_session_discriminators_from_name(&parsed_session.title);
    let relevant_presenter_institution_records = parsed_presenter_institutions
      .iter()
      .filter(|r| r.session_name == session_name_search_query)
      .cloned()
      .collect::<Vec<_>>();

    if !parsed_session.presenters.is_empty()
      && relevant_presenter_institution_records.is_empty()
    {
      warn!(
        session = session_name_search_query,
        "found no presenter-institution-session records with given session"
      );
    }

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
          warn!(
            name = paid_presenter.name,
            session = session_name_search_query,
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

    let relevant_strands_records = parsed_strands
      .iter()
      .filter(|r| r.title == session_name_search_query)
      .cloned()
      .collect::<Vec<_>>();

    if relevant_strands_records.is_empty() {
      warn!(
        session = session_name_search_query,
        "found no strands records with given session title",
      );
    }
    if relevant_strands_records.len() > 1 {
      warn!(
        session = session_name_search_query,
        "found more than one strands record with given session title",
      );
    }

    let record = relevant_strands_records.first();
    let strand = record.map(|r| r.strand.clone());
    let intended_audience = record.map(|r| r.intended_audience.clone());

    debug!(
      ?strand,
      ?intended_audience,
      session = session_name_search_query,
      "found strands for session"
    );

    let synthesized_session = NasupSession {
      start_datetime,
      end_datetime,
      room: parsed_session.room,
      session_type: parsed_session.session_type,
      title: parsed_session.title,
      description: parsed_session.description,
      approved_presenters,
      strand,
      intended_audience,
      rank: parsed_session.row_index as f32,
    };

    debug!("synthesized full session record: {synthesized_session:#?}");

    synthesized_sessions.push(synthesized_session);
  }

  Ok(synthesized_sessions)
}

pub fn strip_session_discriminators_from_name(mut input: &str) -> String {
  let prefixes_to_strip = [
    "A: ", "B: ", "RT 1: ", "RT 2: ", "RT 3: ", "RT 4: ", "RT 5: ", "RT 6: ",
  ];
  loop {
    let mut stripped = false;
    for prefix in prefixes_to_strip {
      if let Some(remainder) = prefix.strip_prefix_of(input.trim_start()) {
        input = remainder;
        stripped = true;
        break;
      }
    }
    if !stripped {
      break;
    }
  }
  input.to_owned()
}
