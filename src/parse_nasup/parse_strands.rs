use std::ops::Index;

use calamine::Data;
use miette::bail;
use tracing::{debug, trace, warn};

use super::parse_model::ParsedNasupStrandAndIntendedAudience;
use crate::{
  fetch_sheet::DecodedWorksheet,
  parse_nasup::find_commas_without_following_whitespace,
};

pub fn parse_nasup_strands_from_worksheet(
  worksheet: DecodedWorksheet,
) -> miette::Result<Vec<ParsedNasupStrandAndIntendedAudience>> {
  let mut strands_and_intended_audience = Vec::new();

  // skip the header
  let iter = worksheet.main.rows().skip(1);

  for row in iter {
    strands_and_intended_audience.push(parse_nasup_strands_from_row(row)?);
  }

  Ok(strands_and_intended_audience)
}

fn parse_nasup_strands_from_row(
  row: &[Data],
) -> miette::Result<ParsedNasupStrandAndIntendedAudience> {
  miette::ensure!(
    !row.is_empty(),
    "failed to parse XLSX row as NASUP presenter-institutions: row is empty"
  );
  miette::ensure!(
    row.len() >= 4,
    format!(
      "failed to parse XLSX row as NASUP presenter-institutions: too few \
       cells: expected >= 4, got {}",
      row.len()
    )
  );

  // session name
  let session_name = match row.index(0) {
    Data::String(sn) => sn.trim().to_owned(),
    d => bail!("session_name column is not a string, got {d:?}"),
  };
  trace!(session_name, "parsed session_name column");

  // author names
  let author_names = match row.index(1) {
    Data::String(an) => an.trim().to_owned(),
    d => bail!("author_names column is not a string, got {d:?}"),
  };
  let commas_without_following_whitespace =
    find_commas_without_following_whitespace(&author_names);
  if !commas_without_following_whitespace.is_empty() {
    warn!(
      author_names,
      "found commas that may be missing a space in author_names column"
    );
  }
  let split_author_names = author_names
    .split(", ")
    .map(ToOwned::to_owned)
    .collect::<Vec<_>>();
  trace!(?split_author_names, "found and split author names");

  // strands
  let strand = match row.index(2) {
    Data::String(an) => an.trim().to_owned(),
    d => bail!("strands column is not a string, got {d:?}"),
  };
  trace!(strand, "parsed strand column");

  // intended_audience
  let intended_audience = match row.index(3) {
    Data::String(an) => an.trim().to_owned(),
    d => bail!("intended_audience column is not a string, got {d:?}"),
  };
  trace!(strand, "parsed intended_audience column");
  let commas_without_following_whitespace =
    find_commas_without_following_whitespace(&intended_audience);
  if !commas_without_following_whitespace.is_empty() {
    warn!(
      intended_audience,
      "found commas that may be missing a space in intended_audience column"
    );
  }
  let split_intended_audience = intended_audience
    .split(", ")
    .map(ToOwned::to_owned)
    .collect::<Vec<_>>();
  trace!(
    ?split_intended_audience,
    "found and split intended_audience"
  );

  let strands_and_intended_audience = ParsedNasupStrandAndIntendedAudience {
    title: session_name,
    presenters: split_author_names,
    strand,
    intended_audience,
  };

  debug!(
    "parsed full strands_and_intended_audience: \
     {strands_and_intended_audience:#?}"
  );

  Ok(strands_and_intended_audience)
}
