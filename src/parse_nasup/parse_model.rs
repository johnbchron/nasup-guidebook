use miette::bail;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ParsedNasupSessionType {
  // #[serde(rename = "Collaborative Conversations")]
  CollaborativeConversations,
  // #[serde(rename = "Extended Practioner Workshop")]
  ExtendedPractionerWorkshop,
  // #[serde(rename = "General")]
  General,
  // #[serde(rename = "Leadership")]
  Leadership,
  // #[serde(rename = "Paired Concurrent")]
  PairedConcurrent(PairedConcurrentDiscriminant),
  // #[serde(rename = "Partnership-Focused Workshop")]
  PartnershipFocusedWorkshop,
  // #[serde(rename = "Practitioner-Focused Workshop")]
  PractitionerFocusedWorkshop,
  // #[serde(rename = "Pre-Conference")]
  PreConference,
  // #[serde(rename = "Preservice Teacher Event")]
  PreServiceTeacherEvent,
  // #[serde(rename = "Round Tables")]
  RoundTable(u8),
  // #[serde(rename = "Symposium")]
  Symposium,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PairedConcurrentDiscriminant {
  A,
  B,
}

impl ParsedNasupSessionType {
  pub fn from_type_and_title(
    session_type: &str,
    session_title: &str,
  ) -> miette::Result<ParsedNasupSessionType> {
    Ok(match session_type {
      "Collaborative Conversations" => {
        ParsedNasupSessionType::CollaborativeConversations
      }
      "Extended Practioner Workshop" => {
        ParsedNasupSessionType::ExtendedPractionerWorkshop
      }
      "General" => ParsedNasupSessionType::General,
      "Leadership" => ParsedNasupSessionType::Leadership,
      "Paired Concurrent" => {
        ParsedNasupSessionType::PairedConcurrent(match &session_title[..3] {
          "A: " => PairedConcurrentDiscriminant::A,
          "B: " => PairedConcurrentDiscriminant::B,
          p => bail!(
            "found unknown session title prefix when looking for a paired \
             concurrent prefix: found prefix: {p:?}"
          ),
        })
      }
      "Partnership-Focused Workshop" => {
        ParsedNasupSessionType::PartnershipFocusedWorkshop
      }
      "Practitioner-Focused Workshop" => {
        ParsedNasupSessionType::PractitionerFocusedWorkshop
      }
      "Pre-Conference" => ParsedNasupSessionType::PreConference,
      "Preservice Teacher Event" => {
        ParsedNasupSessionType::PreServiceTeacherEvent
      }
      "Round Tables" => {
        ParsedNasupSessionType::RoundTable(match &session_title[..6] {
          "RT 1: " => 1,
          "RT 2: " => 2,
          "RT 3: " => 3,
          "RT 4: " => 4,
          "RT 5: " => 5,
          "RT 6: " => 6,
          p => bail!(
            "found unknown session title prefix when looking for a round \
             table prefix: found prefix: {p:?}"
          ),
        })
      }
      "Symposium" => ParsedNasupSessionType::Symposium,
      t => bail!("found unknown session type: found type {t:?}"),
    })
  }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedNasupPresenterWithPaymentStatus {
  pub name: String,
  pub paid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedNasupPresenterWithInstitutionBySession {
  pub name:               String,
  pub session_name:       String,
  pub first_institution:  Option<String>,
  pub second_institution: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParsedNasupStrandsAndIntendedAudience {
  pub title:             String,
  pub presenters:        Vec<String>,
  pub strands:           Vec<String>,
  pub intended_audience: Vec<String>,
}
