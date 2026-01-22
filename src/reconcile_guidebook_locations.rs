use std::collections::HashSet;

use miette::Context;
use tracing::{debug, instrument};

use crate::{
  config::Config,
  guidebook::{Modification, model::GuidebookLocation},
};

#[derive(Debug)]
pub struct LocationsReconciliation {
  pub strands_to_create: Vec<GuidebookLocation>,
}

impl LocationsReconciliation {
  #[instrument(skip(self, config))]
  pub async fn execute_reconciliation(
    &self,
    config: &Config,
  ) -> miette::Result<()> {
    for strand in &self.strands_to_create {
      debug!(
        name = strand.name.as_ref().unwrap(),
        "creating guidebook location"
      );
      crate::guidebook::upsert_guidebook_entity(
        config,
        strand.clone(),
        "/locations/",
        Modification::Create,
      )
      .await
      .context("failed to create location during reconciliation")?;
      debug!(
        name = strand.name.as_ref().unwrap(),
        "successfully created guidebook location"
      );
    }

    Ok(())
  }
}

pub fn reconcile_intended_and_existing_guidebook_locations(
  intended_locations: &[GuidebookLocation],
  existing_locations: &[GuidebookLocation],
) -> miette::Result<LocationsReconciliation> {
  let existing_names = existing_locations
    .iter()
    .map(|l| l.name.as_ref().unwrap())
    .collect::<HashSet<_>>();
  let strands_to_create = intended_locations
    .iter()
    .filter(|l| !existing_names.contains(l.name.as_ref().unwrap()))
    .cloned()
    .collect();

  Ok(LocationsReconciliation { strands_to_create })
}
