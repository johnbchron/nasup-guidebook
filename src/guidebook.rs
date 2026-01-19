pub mod model;

use miette::{Context, IntoDiagnostic};
use serde::Deserialize;
use tracing::{debug, error, instrument, trace, warn};

use self::model::{GuidebookScheduleTrack, GuidebookSession};
use crate::{HTTP_CLIENT, config::Config};

const GUIDEBOOK_BASE_URL: &str = "https://builder.guidebook.com/open-api/v1.1";

#[instrument(skip(config))]
async fn fetch_page_of_guidebook_entities<T: for<'a> Deserialize<'a>>(
  config: &Config,
  url: &str,
) -> miette::Result<model::GuidebookPagedResult<T>> {
  let req = HTTP_CLIENT.get(url).header(
    "Authorization",
    format!("JWT {api_key}", api_key = config.api_key),
  );
  trace!("sending guidebook request to list sessions");
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to fetch guidebook sessions")?
    .error_for_status()
    .into_diagnostic()
    .context(
      "got server error response from response to list guidebook sessions",
    )?;
  trace!(
    content_length = resp.content_length(),
    "got successful response from session listing request"
  );
  let payload = resp
    .json::<model::GuidebookPagedResult<T>>()
    .await
    .into_diagnostic()
    .context("failed to read guidebook session listing response as JSON")?;
  trace!(
    response_count = payload.results.len(),
    total_count = payload.count,
    "parsed session listing response"
  );

  Ok(payload)
}

#[instrument(skip(config))]
pub async fn fetch_all_guidebook_entities<T: for<'a> Deserialize<'a>>(
  config: &Config,
  url: &str,
) -> miette::Result<Vec<T>> {
  let mut results = Vec::new();
  let mut url = format!(
    "{GUIDEBOOK_BASE_URL}{url}/?guide={guide}",
    guide = config.guide_id
  );

  loop {
    let payload = fetch_page_of_guidebook_entities(config, &url)
      .await
      .context("failed to fetch page of guidebook sessions")?;

    results.extend(payload.results);

    if let Some(next_url) = payload.next {
      url = next_url
    } else {
      break;
    }
  }

  debug!(count = results.len(), "fetched guidebook sessions");

  Ok(results)
}

#[derive(Clone, Copy, Debug)]
pub enum SessionModification {
  Create,
  Update,
}

#[instrument(
  skip(config, session),
  fields(session_id = session.id, import_id = session.import_id, url)
)]
pub async fn upsert_guidebook_session(
  config: &Config,
  session: GuidebookSession,
  modification: SessionModification,
) -> miette::Result<GuidebookSession> {
  let url = match modification {
    SessionModification::Create => format!("{GUIDEBOOK_BASE_URL}/sessions/"),
    SessionModification::Update => {
      let session_id = session.id.ok_or(miette::miette!(
        "ID field of guidebook session was unpopulated, cannot form URL"
      ))?;
      format!("{GUIDEBOOK_BASE_URL}/sessions/{session_id}")
    }
  };
  tracing::Span::current().record("url", &url);

  let req = match modification {
    SessionModification::Create => HTTP_CLIENT.post(&url),
    SessionModification::Update => HTTP_CLIENT.patch(&url),
  };
  let req = req
    .header(
      "Authorization",
      format!("JWT {api_key}", api_key = config.api_key),
    )
    .json(&session);

  trace!("sending guidebook request to modify session");
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to modify guidebook session")?;

  // extract error before consuming body
  let server_error = resp
    .error_for_status_ref()
    .map(|_| ())
    .into_diagnostic()
    .context(
      "got server error response from guidebook session modification request",
    );
  let content_length = resp.content_length();
  let payload = resp.text().await.into_diagnostic().context(
    "failed to consume body of response from guidebook session modification \
     request",
  )?;

  // now bubble error
  if let Err(e) = server_error {
    error!(
      payload,
      "got error response from session modification request"
    );
    let () = Err(e)?;
  }
  trace!(
    content_length,
    "got successful response from session modification request"
  );

  let payload = serde_json::from_str::<GuidebookSession>(&payload)
    .into_diagnostic()
    .context(
      "failed to read guidebook session modification response as JSON",
    )?;
  trace!("parsed session modification response");

  Ok(payload)
}

#[instrument(skip(config, schedule_track), fields(url))]
pub async fn create_guidebook_schedule_track(
  config: &Config,
  schedule_track: GuidebookScheduleTrack,
) -> miette::Result<GuidebookScheduleTrack> {
  let url = format!("{GUIDEBOOK_BASE_URL}/schedule-tracks/");
  tracing::Span::current().record("url", &url);

  let req = HTTP_CLIENT
    .post(&url)
    .header(
      "Authorization",
      format!("JWT {api_key}", api_key = config.api_key),
    )
    .json(&schedule_track);

  warn!(
    payload = serde_json::to_string(&schedule_track).unwrap(),
    "sending payload in schedule track creation request"
  );

  trace!("sending guidebook request to create schedule track");
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to create schedule track")?;

  // extract error before consuming body
  let server_error = resp
    .error_for_status_ref()
    .map(|_| ())
    .into_diagnostic()
    .context(
      "got server error response from guidebook schedule track creation \
       request",
    );
  let content_length = resp.content_length();
  let payload = resp.text().await.into_diagnostic().context(
    "failed to consume body of response from guidebook schedule track \
     creation request",
  )?;

  // now bubble error
  if let Err(e) = server_error {
    error!(
      payload,
      "got error response from schedule track creation request"
    );
    let () = Err(e)?;
  }
  trace!(
    content_length,
    "got successful response from schedule track creation request"
  );

  let payload = serde_json::from_str::<GuidebookScheduleTrack>(&payload)
    .into_diagnostic()
    .context(
      "failed to read guidebook schedule track creation response as JSON",
    )?;
  trace!("parsed schedule track creation response");

  Ok(payload)
}
