use miette::Context;
use tracing::{debug, instrument, warn};

use crate::{config::Config, guidebook::model::GuidebookPresenter};

#[derive(Debug)]
pub struct PresenterReconciliation {
  pub presenters_to_create: Vec<GuidebookPresenter>,
  pub presenters_to_update: Vec<GuidebookPresenter>,
}

impl PresenterReconciliation {
  #[instrument(skip(self, config))]
  pub async fn execute_reconciliation(
    &self,
    config: &Config,
  ) -> miette::Result<()> {
    for presenter in &self.presenters_to_create {
      // create presenter
      debug!(
        name = presenter.name.as_ref().unwrap(),
        "creating guidebook presenter"
      );
      let new_presenter = crate::guidebook::upsert_guidebook_entity(
        config,
        presenter.clone(),
        "/custom-list-items/",
        crate::guidebook::Modification::Create,
      )
      .await
      .context("failed to create presenter during reconciliation")?;

      // relate presenter to custom list
      let relation_payload = serde_json::json!({
        "custom_list": config.presenter_custom_list_id,
        "custom_list_item": new_presenter.id.unwrap(),
      });
      crate::guidebook::upsert_guidebook_entity(
        config,
        relation_payload,
        "/custom-list-item-relations/",
        crate::guidebook::Modification::Create,
      )
      .await
      .context(
        "failed to relate new presenter to presenter list during \
         reconciliation",
      )?;
      debug!(
        name = presenter.name.as_ref().unwrap(),
        "successfully created guidebook presenter"
      );
    }

    for presenter in &self.presenters_to_update {
      debug!(?presenter, "updating guidebook presenter");
      crate::guidebook::upsert_guidebook_entity(
        config,
        presenter.clone(),
        "/custom-list-items/",
        crate::guidebook::Modification::Update {
          id: presenter.id.unwrap(),
        },
      )
      .await
      .context("failed to update presenter during reconciliation")?;
      debug!(?presenter, "successfully updated guidebook presenter");
    }

    Ok(())
  }
}

pub fn reconcile_intended_and_existing_guidebook_presenters(
  intended_presenters: &[GuidebookPresenter],
  existing_presenters: &[GuidebookPresenter],
) -> miette::Result<PresenterReconciliation> {
  let mut presenters_to_create = Vec::new();
  let mut presenters_to_update = Vec::new();

  for intended_presenter in intended_presenters {
    // find the existing presenter where the name matches and the subtitle
    // either matches or is supposed to be empty (and is not present in the
    // existing)
    match existing_presenters.iter().find(|ep| {
      ep.name.as_ref() == intended_presenter.name.as_ref()
        && ((ep.subtitle.as_ref() == intended_presenter.subtitle.as_ref())
          || intended_presenter
            .subtitle
            .as_ref()
            .is_some_and(String::is_empty))
    }) {
      Some(existing_presenter) => {
        let patch = GuidebookPresenter::generate_patch_diff(
          intended_presenter,
          existing_presenter,
        );
        if !patch.is_empty_patch() {
          warn!(
            ?intended_presenter,
            ?existing_presenter,
            "generated non-empty patch diff"
          );
          presenters_to_update.push(patch);
        }
      }
      None => {
        presenters_to_create.push(intended_presenter.clone());
      }
    }
  }

  Ok(PresenterReconciliation {
    presenters_to_create,
    presenters_to_update,
  })
}
