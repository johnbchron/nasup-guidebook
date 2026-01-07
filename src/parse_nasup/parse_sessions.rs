use std::{ops::Index, str::FromStr};

use calamine::Data;
use miette::{Context, IntoDiagnostic, bail, miette};
use tracing::{debug, instrument, trace, warn};

use super::parse_model::ParsedNasupSession;
use crate::{
  fetch_sheet::DecodedWorksheet,
  parse_nasup::parse_model::{
    ParsedNasupPresenterWithPaymentStatus, ParsedNasupSessionType,
  },
};

pub fn parse_nasup_sessions_from_worksheet(
  worksheet: DecodedWorksheet,
) -> miette::Result<Vec<ParsedNasupSession>> {
  let mut sessions = Vec::new();

  let iter = worksheet.main.rows().enumerate();
  // skip the header
  let iter = iter.skip(1);

  for (row_index, row) in iter {
    sessions.push(parse_nasup_session_from_row(
      row_index,
      row,
      &worksheet.styles,
    )?);
  }
  Ok(sessions)
}

#[instrument(skip(row, styles))]
pub fn parse_nasup_session_from_row(
  row_index: usize,
  row: &[Data],
  styles: &umya_spreadsheet::Worksheet,
) -> miette::Result<ParsedNasupSession> {
  miette::ensure!(
    !row.is_empty(),
    "failed to parse XLSX row as NASUP session: row is empty"
  );
  miette::ensure!(
    row.len() >= 9,
    format!(
      "failed to parse XLSX row as NASUP session: too few cells: expected >= \
       9, got {}",
      row.len()
    )
  );

  // day of week
  let day_of_week = match row.index(0) {
    Data::String(dow) => dow,
    d => bail!("day-of-week column is not a string, got {d:?}"),
  };
  let day_of_week = chrono::Weekday::from_str(day_of_week)
    .into_diagnostic()
    .context(format!("failed to parse day-of-week, got {day_of_week}"))?;
  trace!(%day_of_week, "parsed day_of_week column");

  // date
  let date = match row.index(1) {
    Data::DateTime(dt) => dt,
    d => bail!("date column is not a date-time, got {d:?}"),
  };
  let (y, m, d, ..) = date.to_ymd_hms_milli();
  let date = chrono::NaiveDate::from_ymd_opt(y as _, m as _, d as _).ok_or(
    miette!("date column is an invalid date: y = {y}, m = {m}, d = {d}"),
  )?;
  trace!(%date, "parsed date column");

  // start time
  let start_time = match row.index(2) {
    Data::DateTime(dt) => dt,
    d => bail!("start_time column is not a date-time, got {d:?}"),
  };
  let (_y, _m, _d, h, m, s, _millis) = start_time.to_ymd_hms_milli();
  let start_time = chrono::NaiveTime::from_hms_opt(h as _, m as _, s as _)
    .ok_or(miette!("start_time column is an invalid time"))?;
  trace!(%start_time, "parsed start_time column");

  // end time
  let end_time = match row.index(3) {
    Data::DateTime(dt) => dt,
    d => bail!("end_time column is not a date-time, got {d:?}"),
  };
  let (_y, _m, _d, h, m, s, _millis) = end_time.to_ymd_hms_milli();
  let end_time = chrono::NaiveTime::from_hms_opt(h as _, m as _, s as _)
    .ok_or(miette!("end_time column is an invalid time"))?;
  trace!(%end_time, "parsed end_time column");

  // room
  let room = match row.index(4) {
    Data::String(r) => r.trim().to_owned(),
    d => bail!("room column is not a string, got {d:?}"),
  };
  trace!(room, "parsed room column");

  // title
  // parsed before type because we need the title for discriminators
  let title = match row.index(6) {
    Data::String(t) => t.trim().to_owned(),
    d => bail!("title column is not a string, got {d:?}"),
  };
  trace!(title, "parsed title column");

  // type
  let session_type = match row.index(5) {
    Data::String(t) => t.trim(),
    d => bail!("session_type column is not a string, got {d:?}"),
  };
  // quoted so that serde_json will parse it as JSON
  let session_type =
    ParsedNasupSessionType::from_type_and_title(session_type, &title).context(
      format!("failed to parse session_type column, got \"{session_type}\""),
    )?;
  trace!(?session_type, "parsed session_type column");

  // description
  let description = match row.index(7) {
    Data::String(d) => d.trim().to_owned(),
    Data::Empty => String::new(),
    d => bail!("description column is not a string, got {d:?}"),
  };
  trace!(description, "parsed description column");

  // presenters
  let presenter_cells = row[8..]
    .iter()
    .enumerate()
    .map(|(i, d)| (i + 8, d))
    .filter(|(_, d)| !matches!(d, Data::Empty))
    .collect::<Vec<_>>();

  let mut presenters = Vec::new();
  for (x, d) in presenter_cells {
    let name = match d {
      Data::String(r) => r.trim().to_owned(),
      d => bail!("presenter name column is not a string, got {d:?}"),
    };

    let coords = (x as u32, row_index as u32);
    // coords are one-indexed in umya :shrug:
    let cell_style = styles.get_style((coords.0 + 1, coords.1 + 1));
    let is_white = match cell_style.get_background_color() {
      Some(color) => {
        if color.get_argb() == "FFFFFFFF" {
          warn!(
            ?coords,
            "color was set for presenter cell, but was still white"
          );
          true
        } else {
          trace!(?coords, ?color, "got color for presenter cell");
          false
        }
      }
      None => {
        trace!(?coords, "found no style for presenter cell");
        true
      }
    };

    presenters.push(ParsedNasupPresenterWithPaymentStatus {
      name,
      paid: !is_white,
    });
  }

  let session = ParsedNasupSession {
    date,
    start_time,
    end_time,
    room,
    session_type,
    title,
    description,
    presenters,
  };

  debug!("parsed full session: {session:#?}");

  Ok(session)
}
