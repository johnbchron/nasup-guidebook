pub mod model;

use miette::{Context, IntoDiagnostic};
use tracing::{debug, instrument, trace};

use crate::{HTTP_CLIENT, config::Config};

const GUIDEBOOK_BASE_URL: &str = "https://builder.guidebook.com/open-api/v1.1";

#[instrument(skip(config))]
async fn fetch_page_of_guidebook_sessions(
  config: &Config,
  url: &str,
) -> miette::Result<model::GuidebookPagedResult<model::GuidebookSession>> {
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
    .json::<model::GuidebookPagedResult<model::GuidebookSession>>()
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
pub async fn fetch_all_guidebook_sessions(
  config: &Config,
) -> miette::Result<Vec<model::GuidebookSession>> {
  let mut results = Vec::new();
  let mut url = format!(
    "{GUIDEBOOK_BASE_URL}/sessions/?guide={guide}",
    guide = config.guide_id
  );

  loop {
    let payload = fetch_page_of_guidebook_sessions(config, &url)
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
