use miette::{Context, IntoDiagnostic};
use tracing::{debug, instrument};

use crate::{
  config::Config,
  guidebook::model::GuidebookSession,
  synth_nasup::{NasupSession, strip_session_discriminators_from_name},
};

#[instrument(skip(config, nasup_session))]
pub fn nasup_session_to_guidebook_session(
  config: &Config,
  nasup_session: NasupSession,
) -> miette::Result<GuidebookSession> {
  let discriminator_stripped_name =
    strip_session_discriminators_from_name(&nasup_session.title);
  let description_html = format!(
    "<h1>{discriminator_stripped_and_escaped_name}</h1>{description_text}",
    discriminator_stripped_and_escaped_name =
      html_escape::encode_text(&discriminator_stripped_name),
    description_text = html_escape::encode_text(&nasup_session.description),
  );

  let session_primary_key = serde_json::json!({
    "type": nasup_session.session_type,
    "start": nasup_session.start_datetime,
    "end": nasup_session.end_datetime,
  });
  let session_primary_key_json = serde_json::to_string(&session_primary_key)
    .into_diagnostic()
    .context("failed to serialize session primary key into JSON")?;

  let session = GuidebookSession {
    id: None,
    guide_id: config.guide_id as u32,
    name: Some(discriminator_stripped_name),
    description_html: Some(description_html),
    start_time: nasup_session.start_datetime,
    end_time: Some(nasup_session.end_datetime),
    all_day: Some(false),
    allow_rating: Some(false),
    add_to_schedule: Some(true),
    import_id: Some(session_primary_key_json.clone()),
    locations: None,
    schedule_tracks: None,
    rank: Some(1.0),
    registration_start_date: None,
    registration_end_date: None,
    require_login: Some(true),
    waitlist: Some(false),
    max_capacity: None,
  };

  debug!(
    primary_key = session_primary_key_json,
    "calculated guidebook session from nasup session"
  );

  Ok(session)
}
