use std::collections::HashMap;

use miette::Context;
use tracing::debug;

use crate::{
  config::Config,
  guidebook::{
    delete_guidebook_entity, fetch_all_guidebook_entities,
    upsert_guidebook_entity,
  },
};

pub async fn synchronize_session_links(
  config: &Config,
  intended_session_to_presenter_links: HashMap<u32, Vec<u32>>,
) -> miette::Result<()> {
  let existing_links =
    fetch_all_guidebook_entities::<serde_json::Value>(config, "/links").await?;

  let mut links_to_delete = Vec::new();
  // from session to presenter
  let mut outbound_links_to_create = Vec::new();
  // from presenter to session
  let mut inbound_links_to_create = Vec::new();

  for (session_id, presenter_ids) in intended_session_to_presenter_links {
    // keep track of the links we still need to make, in and out of this session
    let mut needed_inbound_links = presenter_ids.clone();
    let mut needed_outbound_links = presenter_ids.clone();

    // for all the links out of the session
    for existing_outbound_link in existing_links.iter().filter(|l| {
      l["source_content_type"] == "schedule.session"
        && l["source_object_id"] == session_id
    }) {
      let existing_outbound_link_target =
        existing_outbound_link["target_object_id"].as_u64().unwrap() as u32;
      // if the target isn't one of the given presenters, mark it for deletion
      if !presenter_ids.contains(&existing_outbound_link_target) {
        links_to_delete
          .push(existing_outbound_link["id"].clone().as_u64().unwrap() as u32);
      } else {
        // otherwise mark that we don't need to create it
        needed_outbound_links.retain(|id| *id != existing_outbound_link_target);
      }
    }

    // we need to create outbound links for all the presenters we didn't find
    // existing outbound links for
    outbound_links_to_create.extend(
      needed_outbound_links
        .into_iter()
        .map(|pid| (session_id, pid)),
    );

    // for all the links into the session
    for existing_inbound_link in existing_links.iter().filter(|l| {
      l["target_content_type"] == "schedule.session"
        && l["target_object_id"] == session_id
    }) {
      let existing_inbound_link_source =
        existing_inbound_link["source_object_id"].as_u64().unwrap() as u32;
      // if the source isn't one of the given presenters, mark it for deletion
      if !presenter_ids.contains(&existing_inbound_link_source) {
        links_to_delete
          .push(existing_inbound_link["id"].clone().as_u64().unwrap() as u32);
      } else {
        // otherwise mark that we don't need to create it
        needed_inbound_links.retain(|id| *id != existing_inbound_link_source);
      }
    }

    // we need to create inbound links for all the presenters we didn't find
    // existing inbound links for
    inbound_links_to_create.extend(
      needed_inbound_links
        .into_iter()
        .map(|pid| (pid, session_id)),
    );
  }

  // now we need to delete links
  for link_id in links_to_delete {
    delete_guidebook_entity(config, "/links", link_id)
      .await
      .context("failed to delete link during synchronization")?;
    debug!(link_id, "deleted session link");
  }

  // now to create outbound links
  for (source_session_id, target_presenter_id) in outbound_links_to_create {
    let payload = serde_json::json!({
      "target_object_id": target_presenter_id,
      "source_object_id": source_session_id,
      "source_content_type": "schedule.session",
      "guide": config.guide_id,
      "target_content_type": "custom_list.customlistitem"
    });
    upsert_guidebook_entity(
      config,
      payload,
      "/links/",
      crate::guidebook::Modification::Create,
    )
    .await
    .context(
      "failed to create outbound link from session during synchronization",
    )?;
    debug!(
      source_session_id,
      target_presenter_id, "added outbound link from session to presenter"
    );
  }

  // now to create inbound links
  for (source_presenter_id, target_session_id) in inbound_links_to_create {
    let payload = serde_json::json!({
      "target_object_id": target_session_id,
      "source_object_id": source_presenter_id,
      "source_content_type": "custom_list.customlistitem",
      "guide": config.guide_id,
      "target_content_type": "schedule.session"
    });
    upsert_guidebook_entity(
      config,
      payload,
      "/links/",
      crate::guidebook::Modification::Create,
    )
    .await
    .context(
      "failed to create outbound link from session during synchronization",
    )?;
    debug!(
      source_presenter_id,
      target_session_id, "added inbound link from presenter to session"
    );
  }

  Ok(())
}
