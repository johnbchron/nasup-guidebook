use std::{collections::HashMap, ops::Index};

use calamine::Data;
use miette::bail;
use tracing::{debug, trace, warn};

use super::parse_model::ParsedNasupPresenterWithInstitutionBySession;
use crate::fetch_sheet::DecodedWorksheet;

pub fn parse_nasup_presenter_institutions_from_worksheet(
  worksheet: DecodedWorksheet,
) -> miette::Result<Vec<ParsedNasupPresenterWithInstitutionBySession>> {
  let mut presenter_institutions = Vec::new();

  let iter = worksheet.main.rows().enumerate();
  // skip the header
  let iter = iter.skip(1);

  for (row_index, row) in iter {
    presenter_institutions.extend(
      parse_nasup_presenter_institutions_from_row(row_index, row)?.into_iter(),
    );
  }

  Ok(presenter_institutions)
}

fn parse_nasup_presenter_institutions_from_row(
  _row_index: usize,
  row: &[Data],
) -> miette::Result<Vec<ParsedNasupPresenterWithInstitutionBySession>> {
  miette::ensure!(
    !row.is_empty(),
    "failed to parse XLSX row as NASUP presenter-institutions: row is empty"
  );
  miette::ensure!(
    row.len() >= 9,
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
  let split_author_names = author_names.split(", ").collect::<Vec<_>>();
  trace!(?split_author_names, "found and split author names");

  // column C is blank

  // following columns are author 1 org 1, author 1 org 2, author 2 org 1,
  // author 2 org 2, author 3 org 1, etc.
  let mut institutions = HashMap::new();
  row[3..].iter().enumerate().try_for_each(|(i, d)| match d {
    Data::String(s) if s.is_empty() => Ok(()),
    Data::String(inst) => {
      institutions.insert((i.div_floor(2), i.rem_euclid(2)), inst.to_owned());
      Ok(())
    }
    Data::Empty => Ok(()),
    d => bail!("institution column is not a string or empty, got {d:?}"),
  })?;
  trace!(?institutions, "parsed institution columns");

  let results = split_author_names
    .into_iter()
    .enumerate()
    .map(|(i, name)| {
      let record = ParsedNasupPresenterWithInstitutionBySession {
        name:               name.to_owned(),
        session_name:       session_name.clone(),
        first_institution:  institutions.remove(&(i, 0)),
        second_institution: institutions.remove(&(i, 1)),
      };
      debug!("collected author-institution-session record: {record:#?}");
      record
    })
    .collect();

  if !institutions.is_empty() {
    warn!(
      ?institutions,
      "some institutions were not matched with author names"
    );
  }

  Ok(results)
}

fn find_commas_without_following_whitespace(text: &str) -> Vec<usize> {
  text
    .char_indices()
    .zip(text.chars().skip(1))
    .filter_map(|((i, c), next)| {
      if c == ',' && !next.is_whitespace() {
        Some(i)
      } else {
        None
      }
    })
    .collect()
}
