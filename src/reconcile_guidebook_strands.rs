use std::collections::HashSet;

use miette::Context;
use tracing::{debug, instrument};

use crate::{config::Config, guidebook::model::GuidebookScheduleTrack};

#[derive(Debug)]
pub struct StrandsReconciliation {
  pub strands_to_create: Vec<GuidebookScheduleTrack>,
}

impl StrandsReconciliation {
  #[instrument(skip(self, config))]
  pub async fn execute_reconciliation(
    &self,
    config: &Config,
  ) -> miette::Result<()> {
    for strand in &self.strands_to_create {
      debug!(
        name = strand.name.as_ref().unwrap(),
        "creating guidebook schedule track"
      );
      crate::guidebook::create_guidebook_schedule_track(config, strand.clone())
        .await
        .context("failed to create schedule track during reconciliation")?;
      debug!(
        name = strand.name.as_ref().unwrap(),
        "successfully created guidebook schedule track"
      );
    }

    Ok(())
  }
}

pub fn reconcile_intended_and_existing_guidebook_schedule_tracks(
  intended_schedule_tracks: &[GuidebookScheduleTrack],
  existing_schedule_tracks: &[GuidebookScheduleTrack],
) -> miette::Result<StrandsReconciliation> {
  let existing_names = existing_schedule_tracks
    .iter()
    .map(|st| st.name.as_ref().unwrap())
    .collect::<HashSet<_>>();
  let strands_to_create = intended_schedule_tracks
    .iter()
    .filter(|st| !existing_names.contains(st.name.as_ref().unwrap()))
    .cloned()
    .collect();

  Ok(StrandsReconciliation { strands_to_create })
}
