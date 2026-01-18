use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuidebookPagedResult<T> {
  pub count:    usize,
  pub next:     Option<String>,
  pub previous: Option<String>,
  pub results:  Vec<T>,
}

fn patch_field<T: std::fmt::Debug + Clone + PartialEq>(
  intended: &Option<T>,
  existing: &Option<T>,
) -> Option<T> {
  match (intended, existing) {
    // not populated, so don't set
    (None, _) => None,
    // populated, so set
    (Some(int), None) => Some(int.clone()),
    // populated and correct, so don't set
    (Some(int), Some(exi)) if int == exi => None,
    // populated and incorrect, so set
    (Some(int), Some(exi)) => {
      warn!(intended = ?int, existing = ?exi, "patching incorrect field");
      Some(int.clone())
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuidebookSession {
  /// The ID of the `Session`
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<u32>,
  /// The specific `Guide` your `Session` belongs to.
  #[serde(rename = "guide")]
  pub guide_id: u32,
  /// The title of your `Session`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// A text description of the `Session`. This field has a 20,000 character
  /// limit. This field supports basic HTML.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description_html: Option<String>,
  /// The start time of the event. For consistency, all timestamps are
  /// converted to the UTC timezone.
  pub start_time: DateTime<Utc>,
  /// The end time of the event. Leave blank for all day events. For
  /// consistency, all timestamps are converted to the UTC timezone.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_time: Option<DateTime<Utc>>,
  /// A boolean value indicating if a `Session` runs for the entire day.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub all_day: Option<bool>,
  /// A boolean value indicating if end-users can rate this `Session`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_rating: Option<bool>,
  /// A boolean value indicating if end-users can add this `Session` to their
  /// personal schedule.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub add_to_schedule: Option<bool>,
  /// A string field you can use to input your own identifier. This is for
  /// when you have your own IDs for `Session`s in your data store.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub import_id: Option<String>,
  /// Array of IDs of `Location`s this `Session` should belong to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locations: Option<Vec<u32>>,
  /// Array of IDs of `ScheduleTracks` this `Session` should belong to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub schedule_tracks: Option<Vec<u32>>,
  /// The order the `Session` will appear.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rank: Option<f32>,
  /// The date from which users can start registering or add the current
  /// session to their personal schedule. Setting this field requires that
  /// `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_start_date: Option<DateTime<Utc>>,
  /// The date when users can no longer register or add the current session
  /// to their personal schedule. Setting this field requires that
  /// `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_end_date: Option<DateTime<Utc>>,
  /// A boolean value indicating if a user needs to be logged in to add this
  /// `Session` to their schedule. Setting this field requires that
  /// `add_to_schedule` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub require_login: Option<bool>,
  /// A boolean value indicating if this `Session` should have a registration
  /// waitlist. This field requires that `max_capacity` is set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub waitlist: Option<bool>,
  /// The number of people who can add this session. Setting this field
  /// requires that `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_capacity: Option<u32>,
}

impl GuidebookSession {
  pub fn is_empty_patch(&self) -> bool {
    // not comparing id, guide_id, start_time, or import_id
    self.name.is_none()
      && self.description_html.is_none()
      && self.end_time.is_none()
      && self.all_day.is_none()
      && self.allow_rating.is_none()
      && self.add_to_schedule.is_none()
      && self.locations.is_none()
      && self.schedule_tracks.is_none()
      && self.rank.is_none()
      && self.registration_start_date.is_none()
      && self.registration_end_date.is_none()
      && self.require_login.is_none()
      && self.waitlist.is_none()
      && self.max_capacity.is_none()
  }

  pub fn generate_patch_diff(intended: &Self, existing: &Self) -> Self {
    Self {
      // ID can't be updated, and the difference here isn't meaningful
      id: None,
      // guide_id can't be updated, and shouldn't
      guide_id: existing.guide_id,
      name: patch_field(&intended.name.clone(), &existing.name.clone()),
      description_html: patch_field(
        &intended.description_html,
        &existing.description_html,
      ),
      start_time: intended.start_time,
      end_time: patch_field(&intended.end_time, &existing.end_time),
      all_day: patch_field(&intended.all_day, &existing.all_day),
      allow_rating: patch_field(&intended.allow_rating, &existing.allow_rating),
      add_to_schedule: patch_field(
        &intended.add_to_schedule,
        &existing.add_to_schedule,
      ),
      // guide_id can't be updated, and shouldn't
      import_id: existing.import_id.clone(),
      locations: patch_field(&intended.locations, &existing.locations),
      schedule_tracks: patch_field(
        &intended.schedule_tracks,
        &existing.schedule_tracks,
      ),
      rank: patch_field(&intended.rank, &existing.rank),
      registration_start_date: patch_field(
        &intended.registration_start_date,
        &existing.registration_start_date,
      ),
      registration_end_date: patch_field(
        &intended.registration_end_date,
        &existing.registration_end_date,
      ),
      require_login: patch_field(
        &intended.require_login,
        &existing.require_login,
      ),
      waitlist: patch_field(&intended.waitlist, &existing.waitlist),
      max_capacity: patch_field(&intended.max_capacity, &existing.max_capacity),
    }
  }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuidebookPresenter {
  /// The ID of the `Session`
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id:               Option<u32>,
  /// The specific `Guide` your `Session` belongs to.
  #[serde(rename = "guide")]
  pub guide_id:         u32,
  /// The title of your `Session`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name:             Option<String>,
  /// A text description of the `Session`. This field has a 20,000 character
  /// limit. This field supports basic HTML.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description_html: Option<String>,
  /// A short tagline thats displayed below the name of the name field.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subtitle:         Option<String>,
  /// A boolean value indicating if end-users can rate this `Session`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_rating:     Option<bool>,
  /// A string field you can use to input your own identifier. This is for
  /// when you have your own IDs for `Session`s in your data store.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub import_id:        Option<String>,
  /// Array of IDs of `Location`s this `Session` should belong to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locations:        Option<Vec<u32>>,
  /// An email for the item that users will be able to contact directly from
  /// the app or Guidebook Web.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub contact_email:    Option<String>,
}

impl GuidebookPresenter {
  pub fn is_empty_patch(&self) -> bool {
    // not comparing id, guide_id, or import_id
    self.name.is_none()
      && self.description_html.is_none()
      && self.subtitle.is_none()
      && self.allow_rating.is_none()
      && self.locations.is_none()
      && self.contact_email.is_none()
  }

  pub fn generate_patch_diff(intended: &Self, existing: &Self) -> Self {
    Self {
      // ID can't be updated, and the difference here isn't meaningful
      id:               None,
      // guide_id can't be updated, and shouldn't
      guide_id:         existing.guide_id,
      name:             patch_field(
        &intended.name.clone(),
        &existing.name.clone(),
      ),
      description_html: patch_field(
        &intended.description_html,
        &existing.description_html,
      ),
      subtitle:         patch_field(&intended.subtitle, &existing.subtitle),
      allow_rating:     patch_field(
        &intended.allow_rating,
        &existing.allow_rating,
      ),
      // guide_id can't be updated, and shouldn't
      import_id:        existing.import_id.clone(),
      locations:        patch_field(&intended.locations, &existing.locations),
      contact_email:    patch_field(
        &intended.contact_email,
        &existing.contact_email,
      ),
    }
  }
}
