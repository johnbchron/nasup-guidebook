pub mod model;

use miette::{Context, IntoDiagnostic};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, instrument, trace};

use self::model::GuidebookScheduleTrack;
use crate::{HTTP_CLIENT, config::Config};

const GUIDEBOOK_BASE_URL: &str = "https://builder.guidebook.com/open-api/v1.1";

#[instrument(skip(config))]
pub async fn delete_guidebook_entity(
  config: &Config,
  url: &str,
  id: u32,
) -> miette::Result<()> {
  let url =
    format!("{GUIDEBOOK_BASE_URL}{url}/{id}", url = url.trim_suffix("/"));
  let req = HTTP_CLIENT
    .delete(url)
    .header(
      "Authorization",
      format!("JWT {api_key}", api_key = config.api_key),
    )
    .query(&[("guide", &config.guide_id.to_string())]);

  trace!("sending guidebook request to delete entity");
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to delete guidebook entity")?
    .error_for_status()
    .into_diagnostic()
    .context(
      "got server error response from response to delete guidebook entity",
    )?;
  trace!(
    content_length = resp.content_length(),
    "got successful response from entity deletion request"
  );

  Ok(())
}

#[instrument(skip(config))]
async fn fetch_page_of_guidebook_entities<T: for<'a> Deserialize<'a>>(
  config: &Config,
  url: &str,
) -> miette::Result<model::GuidebookPagedResult<T>> {
  let req = HTTP_CLIENT
    .get(url)
    .header(
      "Authorization",
      format!("JWT {api_key}", api_key = config.api_key),
    )
    .query(&[("guide", &config.guide_id.to_string())]);

  trace!("sending guidebook request to list entities");
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to fetch guidebook entities")?
    .error_for_status()
    .into_diagnostic()
    .context(
      "got server error response from response to list guidebook entities",
    )?;
  trace!(
    content_length = resp.content_length(),
    "got successful response from entity listing request"
  );

  let payload = resp
    .text()
    .await
    .into_diagnostic()
    .context("failed to read guidebook entity listing response body")?;

  let jd = &mut serde_json::Deserializer::from_str(&payload);
  let payload: model::GuidebookPagedResult<T> =
    serde_path_to_error::deserialize(jd)
      .into_diagnostic()
      .context("failed to parse guidebook entity listing response body as type")
      .inspect_err(|_| {
        error!(
          payload,
          "failed to parse guidebook entity listing response body as type"
        );
      })?;
  trace!(
    response_count = payload.results.len(),
    total_count = payload.count,
    "parsed entity listing response"
  );

  Ok(payload)
}

#[instrument(skip(config))]
pub async fn fetch_all_guidebook_entities<T: for<'a> Deserialize<'a>>(
  config: &Config,
  url: &str,
) -> miette::Result<Vec<T>> {
  let mut results = Vec::new();
  let mut url = format!("{GUIDEBOOK_BASE_URL}{url}");

  loop {
    let payload = fetch_page_of_guidebook_entities(config, &url)
      .await
      .context("failed to fetch page of guidebook entities")?;

    results.extend(payload.results);

    if let Some(next_url) = payload.next {
      url = next_url
    } else {
      break;
    }
  }

  debug!(count = results.len(), "fetched guidebook entities");

  Ok(results)
}

#[derive(Clone, Copy, Debug)]
pub enum Modification {
  Create,
  Update { id: u32 },
}

#[instrument(skip(config, entity), fields(url))]
pub async fn upsert_guidebook_entity<T: Serialize + for<'a> Deserialize<'a>>(
  config: &Config,
  entity: T,
  url: &str,
  modification: Modification,
) -> miette::Result<T> {
  let url = match modification {
    Modification::Create => format!("{GUIDEBOOK_BASE_URL}{url}"),
    Modification::Update { id } => {
      format!("{GUIDEBOOK_BASE_URL}{url}/{id}", url = url.trim_suffix("/"))
    }
  };
  tracing::Span::current().record("url", &url);

  let req = match modification {
    Modification::Create => HTTP_CLIENT.post(&url),
    Modification::Update { id: _ } => HTTP_CLIENT.patch(&url),
  };
  let req = req
    .header(
      "Authorization",
      format!("JWT {api_key}", api_key = config.api_key),
    )
    .query(&[("guide", &config.guide_id.to_string())])
    .json(&entity);

  trace!(
    payload = serde_json::to_string(&entity).unwrap(),
    "sending guidebook request to modify entity"
  );
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request to modify guidebook entity")?;

  // extract error before consuming body
  let server_error = resp
    .error_for_status_ref()
    .map(|_| ())
    .into_diagnostic()
    .context(
      "got server error response from guidebook entity modification request",
    );
  let content_length = resp.content_length();
  let payload = resp.text().await.into_diagnostic().context(
    "failed to consume body of response from guidebook entity modification \
     request",
  )?;

  // now bubble error
  if let Err(e) = server_error {
    error!(
      payload,
      "got error response from entity modification request"
    );
    let () = Err(e)?;
  }
  trace!(
    content_length,
    "got successful response from entity modification request"
  );

  let payload = serde_json::from_str::<T>(&payload)
    .into_diagnostic()
    .context("failed to read guidebook entity modification response as JSON")
    .inspect_err(|_| {
      error!(
        payload,
        "failed to deserialize payload of entity modification response"
      );
    })?;
  trace!("parsed entity modification response");

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
