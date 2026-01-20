use std::{
  collections::HashSet,
  hash::{DefaultHasher, Hash, Hasher},
};

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
          c: if s { 0.16 } else { 0.05 },
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

#[derive(Clone, Debug)]
pub struct WithLinks<T>(pub T, pub Vec<u32>);

#[instrument(skip(
  config,
  nasup_session,
  schedule_tracks,
  existing_presenters
))]
pub fn nasup_session_to_guidebook_session(
  config: &Config,
  nasup_session: NasupSession,
  schedule_tracks: &[GuidebookScheduleTrack],
  existing_presenters: &[GuidebookPresenter],
) -> miette::Result<WithLinks<GuidebookSession>> {
  let discriminator_stripped_name =
    strip_session_discriminators_from_name(&nasup_session.title);
  let description_html = format!(
    "<h1>{discriminator_stripped_and_escaped_name}</h1>{description_text}",
    discriminator_stripped_and_escaped_name =
      html_escape::encode_text(&discriminator_stripped_name),
    description_text = html_escape::encode_text(&nasup_session.description),
  );

  let session_primary_key = nasup_session.primary_key();

  let schedule_tracks_to_find = nasup_session
    .strands
    .into_iter()
    .chain(nasup_session.intended_audience.into_iter());
  let schedule_track_ids = schedule_tracks_to_find
    .filter_map(|stn| {
      match schedule_tracks
        .iter()
        .find(|st| st.name.as_ref().unwrap() == &stn)
      {
        Some(st) => Some(st.clone()),
        None => {
          warn!(name = stn, "failed to find schedule track with name");
          None
        }
      }
    })
    .map(|st| {
      st.id
        .expect("expected schedule track to have its name field")
    })
    .collect();

  let presenters_to_link_to =
    nasup_session.approved_presenters.iter().flat_map(|ap| {
      let intended_presenter =
        nasup_presenter_to_guidebook_presenter(config, ap.clone()).unwrap();
      let located_presenter = existing_presenters.iter().find(|ep| {
        ep.name.as_ref() == intended_presenter.name.as_ref()
          && ((ep.subtitle.as_ref() == intended_presenter.subtitle.as_ref())
            || intended_presenter
              .subtitle
              .as_ref()
              .is_some_and(String::is_empty))
      });
      let Some(id) = located_presenter.map(|p| p.id.unwrap()) else {
        warn!(name = ?intended_presenter.name, subtitle = ?intended_presenter.subtitle, "could not find existing presenter to link to for session");
        return None;
      };
      Some(id)
    }).collect();

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
    import_id: Some(session_primary_key.clone()),
    locations: None,
    schedule_tracks: Some(schedule_track_ids),
    rank: Some(1.0),
    registration_start_date: None,
    registration_end_date: None,
    require_login: Some(true),
    waitlist: Some(false),
    max_capacity: None,
  };

  debug!(
    primary_key = session_primary_key,
    "calculated guidebook session from nasup session"
  );

  Ok(WithLinks(session, presenters_to_link_to))
}

pub fn nasup_sessions_to_guidebook_presenters(
  config: &Config,
  nasup_sessions: &[NasupSession],
) -> miette::Result<Vec<GuidebookPresenter>> {
  nasup_sessions
    .iter()
    .flat_map(|s| s.approved_presenters.clone().into_iter())
    .collect::<HashSet<_>>()
    .into_iter()
    .map(|p| nasup_presenter_to_guidebook_presenter(config, p))
    .try_collect::<Vec<_>>()
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
    description_html: Some("".to_owned()),
    subtitle:         Some(subtitle),
    allow_rating:     None,
    import_id:        None,
    locations:        None,
    contact_email:    None,
  };

  Ok(presenter)
}
