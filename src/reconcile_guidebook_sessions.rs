use std::collections::{HashMap, HashSet};

use miette::Context;
use tracing::{debug, instrument, warn};

use crate::{config::Config, guidebook::model::GuidebookSession};

#[derive(Debug)]
pub struct SessionReconciliation {
  pub orphaned_existing_import_ids: HashSet<String>,
  pub sessions_to_create:           Vec<GuidebookSession>,
  pub sessions_to_update:           Vec<GuidebookSession>,
}

impl SessionReconciliation {
  #[instrument(skip(self, config))]
  pub async fn execute_reconciliation(
    &self,
    config: &Config,
  ) -> miette::Result<()> {
    for session in &self.sessions_to_create {
      debug!(
        import_id = session.import_id.as_ref().unwrap(),
        "creating guidebook session"
      );
      crate::guidebook::upsert_guidebook_session(
        config,
        session.clone(),
        crate::guidebook::SessionModification::Create,
      )
      .await
      .context("failed to create session during reconciliation")?;
      debug!(
        import_id = session.import_id.as_ref().unwrap(),
        "successfully created guidebook session"
      );
    }

    for session in &self.sessions_to_update {
      debug!(
        import_id = session.import_id.as_ref().unwrap(),
        "updating guidebook session"
      );
      crate::guidebook::upsert_guidebook_session(
        config,
        session.clone(),
        crate::guidebook::SessionModification::Update,
      )
      .await
      .context("failed to update session during reconciliation")?;
      debug!(
        import_id = session.import_id.as_ref().unwrap(),
        "successfully updated guidebook session"
      );
    }

    Ok(())
  }
}

pub fn reconcile_intended_and_existing_guidebook_sessions(
  intended_sessions: &[GuidebookSession],
  existing_sessions: &[GuidebookSession],
) -> miette::Result<SessionReconciliation> {
  // build hashmap of existing sessions by import_id
  // warn if no import_id
  let mut existing_sessions_by_import_id = HashMap::new();
  for existing in existing_sessions {
    match &existing.import_id {
      Some(import_id) => {
        existing_sessions_by_import_id
          .insert(import_id.clone(), existing.clone());
      }
      None => {
        warn!(
          session = ?existing,
          "session does not have an import_id, orphaning"
        );
      }
    }
  }

  // hashset of import_ids of existing sessions
  let existing_import_ids = existing_sessions_by_import_id
    .keys()
    .cloned()
    .collect::<HashSet<_>>();

  // build hashmap of intended sessions by import_id
  let mut intended_sessions_by_import_id = HashMap::new();
  for intended in intended_sessions {
    intended_sessions_by_import_id
      .insert(intended.import_id.clone().unwrap(), intended.clone());
  }

  // hashset of import_ids of intended sessions
  let intended_import_ids = intended_sessions_by_import_id
    .keys()
    .cloned()
    .collect::<HashSet<_>>();

  // sessions which exist but are not intended
  let orphaned_existing_import_ids = existing_import_ids
    .difference(&intended_import_ids)
    .cloned()
    .collect::<HashSet<_>>();
  for id in &orphaned_existing_import_ids {
    warn!(
      import_id = id,
      session_id = existing_sessions_by_import_id.get(id).unwrap().id.unwrap(),
      "import ID exists but is not found in intended list, may be getting \
       orphaned"
    );
  }

  // sessions which are intended but do not exist
  let sessions_to_create = intended_import_ids
    .difference(&existing_import_ids)
    .map(|iid| intended_sessions_by_import_id.get(iid).unwrap().clone())
    .collect::<Vec<_>>();

  // sessions which both exist and are intended
  let sessions_to_update = intended_import_ids
    .intersection(&existing_import_ids)
    .map(|iid| {
      let intended = intended_sessions_by_import_id.get(iid).unwrap();
      let existing = existing_sessions_by_import_id.get(iid).unwrap();
      let mut patch_session =
        GuidebookSession::generate_patch_diff(intended, existing);
      patch_session.id = existing.id;
      patch_session
    })
    .filter(|s| !s.is_empty_patch())
    .collect::<Vec<_>>();

  Ok(SessionReconciliation {
    orphaned_existing_import_ids,
    sessions_to_create,
    sessions_to_update,
  })
}
