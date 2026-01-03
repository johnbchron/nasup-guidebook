use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum NasupSessionType {
  #[serde(rename = "Collaborative Conversations")]
  CollaborativeConversations,
  #[serde(rename = "Extended Practioner Workshop")]
  ExtendedPractionerWorkshop,
  #[serde(rename = "General")]
  General,
  #[serde(rename = "Leadership")]
  Leadership,
  #[serde(rename = "Paired Concurrent")]
  PairedConcurrent,
  #[serde(rename = "Partnership-Focused Workshop")]
  PartnershipFocusedWorkshop,
  #[serde(rename = "Practitioner-Focused Workshop")]
  PractitionerFocusedWorkshop,
  #[serde(rename = "Pre-Conference")]
  PreConference,
  #[serde(rename = "Preservice Teacher Event")]
  PreServiceTeacherEvent,
  #[serde(rename = "Round Tables")]
  RoundTables,
  #[serde(rename = "Symposium")]
  Symposium,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NasupPresenter {
  pub name: String,
  pub paid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NasupSession {
  pub date:         chrono::NaiveDate,
  pub start_time:   chrono::NaiveTime,
  pub end_time:     chrono::NaiveTime,
  pub room:         String,
  pub session_type: NasupSessionType,
  pub title:        String,
  pub description:  String,
  pub presenters:   Vec<NasupPresenter>,
}
