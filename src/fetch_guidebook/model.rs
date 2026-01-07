use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GuidebookPagedResult<T> {
  pub count:    usize,
  pub next:     Option<String>,
  pub previous: Option<String>,
  pub results:  Vec<T>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GuidebookSession {
  /// The specific `Guide` your `Session` belongs to.
  #[serde(rename = "guide")]
  pub guide_id:                u32,
  /// The title of your `Session`.
  pub name:                    String,
  /// A text description of the `Session`. This field has a 20,000 character
  /// limit. This field supports basic HTML.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub description_html:        Option<String>,
  /// The start time of the event. For consistency, all timestamps are
  /// converted to the UTC timezone.
  pub start_time:              DateTime<Utc>,
  /// The end time of the event. Leave blank for all day events. For
  /// consistency, all timestamps are converted to the UTC timezone.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub end_time:                Option<DateTime<Utc>>,
  /// A boolean value indicating if a `Session` runs for the entire day.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub all_day:                 Option<bool>,
  /// A boolean value indicating if end-users can rate this `Session`.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub allow_rating:            Option<bool>,
  /// A boolean value indicating if end-users can add this `Session` to their
  /// personal schedule.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub add_to_schedule:         Option<bool>,
  /// A string field you can use to input your own identifier. This is for
  /// when you have your own IDs for `Session`s in your data store.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub import_id:               Option<String>,
  /// Array of IDs of `Location`s this `Session` should belong to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub locations:               Option<Vec<u32>>,
  /// Array of IDs of `ScheduleTracks` this `Session` should belong to.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub schedule_tracks:         Option<Vec<u32>>,
  /// The order the `Session` will appear.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub rank:                    Option<f32>,
  /// The date from which users can start registering or add the current
  /// session to their personal schedule. Setting this field requires that
  /// `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_start_date: Option<DateTime<Utc>>,
  /// The date when users can no longer register or add the current session
  /// to their personal schedule. Setting this field requires that
  /// `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub registration_end_date:   Option<DateTime<Utc>>,
  /// A boolean value indicating if a user needs to be logged in to add this
  /// `Session` to their schedule. Setting this field requires that
  /// `add_to_schedule` is true.
  pub require_login:           Option<bool>,
  /// A boolean value indicating if this `Session` should have a registration
  /// waitlist. This field requires that `max_capacity` is set.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub waitlist:                Option<bool>,
  /// The number of people who can add this session. Setting this field
  /// requires that `require_login` is true.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_capacity:            Option<u32>,
}
