use std::{
  collections::HashSet,
  hash::{DefaultHasher, Hash, Hasher},
};

use miette::{Context, IntoDiagnostic};
use tracing::{debug, instrument, warn};

use crate::{
  config::Config,
  guidebook::model::{
    GuidebookPresenter, GuidebookScheduleTrack, GuidebookSession,
  },
  synth_nasup::{
    NasupPresenter, NasupSession, strip_session_discriminators_from_name,
  },
};

pub fn nasup_sessions_to_guidebook_schedule_tracks(
  config: &Config,
  nasup_sessions: &[NasupSession],
) -> miette::Result<Vec<GuidebookScheduleTrack>> {
  let strands = nasup_sessions
    .iter()
    .flat_map(|s| s.strands.clone().into_iter())
    .collect::<HashSet<_>>();
  let intended_audiences = nasup_sessions
    .iter()
    .flat_map(|s| s.intended_audience.clone().into_iter())
    .collect::<HashSet<_>>();

  Ok(
    strands
      .into_iter()
      .map(|n| (true, n))
      .chain(intended_audiences.into_iter().map(|n| (false, n)))
      .map(|(s, n)| {
        let mut hasher = DefaultHasher::new();
        n.hash(&mut hasher);
        let name_hash = hasher.finish();
        let hue = name_hash as f64 / (u64::MAX as f64 / 360.0);
        let color = colorutils_rs::Oklch {
          l: 0.7,
          c: if s { 0.14 } else { 0.07 },
          h: hue as f32,
        }
        .to_rgb(colorutils_rs::TransferFunction::Srgb);
        let color = format!(
          "#{r:02X}{g:02X}{b:02X}",
          r = color.r,
          g = color.g,
          b = color.b
        );

        GuidebookScheduleTrack {
          id:               None,
          guide_id:         config.guide_id as u32,
          name:             Some(n.clone()),
          description_html: None,
          color:            Some(color),
          import_id:        None,
        }
      })
      .collect(),
  )
}

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

pub fn nasup_presenter_to_guidebook_presenter(
  config: &Config,
  nasup_presenter: NasupPresenter,
) -> miette::Result<GuidebookPresenter> {
  let subtitle = nasup_presenter
    .first_institution
    .iter()
    .chain(nasup_presenter.second_institution.iter())
    .cloned()
    .intersperse(", ".to_owned())
    .collect::<String>();

  let presenter = GuidebookPresenter {
    id:               None,
    guide_id:         config.guide_id as u32,
    name:             Some(nasup_presenter.name),
    description_html: None,
    subtitle:         Some(subtitle),
    allow_rating:     None,
    import_id:        None,
    locations:        None,
    contact_email:    None,
  };

  Ok(presenter)
}
