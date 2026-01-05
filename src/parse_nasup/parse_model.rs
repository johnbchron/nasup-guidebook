use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ParsedNasupSessionType {
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
pub struct ParsedNasupPresenterWithPaymentStatus {
  pub name: String,
  pub paid: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedNasupPresenterWithInstitutionBySession {
  pub name:               String,
  pub session_name:       String,
  pub first_institution:  Option<String>,
  pub second_institution: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedNasupSession {
  pub date:         chrono::NaiveDate,
  pub start_time:   chrono::NaiveTime,
  pub end_time:     chrono::NaiveTime,
  pub room:         String,
  pub session_type: ParsedNasupSessionType,
  pub title:        String,
  pub description:  String,
  pub presenters:   Vec<ParsedNasupPresenterWithPaymentStatus>,
}
