use std::io::Cursor;

use bytes::Bytes;
use calamine::{Data, Range, Reader, Xlsx};
use miette::{Context, IntoDiagnostic};
use tracing::{debug, instrument};

use crate::HTTP_CLIENT;

pub struct DecodedSpreadsheet {
  pub main:   Xlsx<Cursor<Bytes>>,
  pub styles: umya_spreadsheet::Spreadsheet,
}

impl DecodedSpreadsheet {
  pub fn get_worksheet(
    &mut self,
    name: &str,
  ) -> miette::Result<DecodedWorksheet> {
    let main = self
      .main
      .worksheet_range(name)
      .into_diagnostic()
      .context(format!("failed to find sheet in main: \"{name}\""))?;
    self.styles.read_sheet_by_name(name);
    let styles = self
      .styles
      .get_sheet_by_name(name)
      .ok_or(miette::miette!(
        "failed to find sheet in styles: \"{name}\""
      ))?
      .clone();

    Ok(DecodedWorksheet {
      main,
      styles: Box::new(styles),
    })
  }
}

pub struct DecodedWorksheet {
  pub main:   Range<Data>,
  pub styles: Box<umya_spreadsheet::Worksheet>,
}

#[instrument]
pub(crate) async fn fetch_xlsx_from_google_sheets(
  sheet_id: &str,
) -> miette::Result<DecodedSpreadsheet> {
  let url = format!(
    "https://docs.google.com/spreadsheets/d/{sheet_id}/export?format=xlsx"
  );

  debug!("requesting XLSX sheet export");
  let req = HTTP_CLIENT.get(url);
  let resp = req
    .send()
    .await
    .into_diagnostic()
    .context("failed to send request for XLSX export")?
    .error_for_status()
    .into_diagnostic()
    .context("got error response from google for XLSX export")?;
  let body = resp
    .bytes()
    .await
    .into_diagnostic()
    .context("failed to read full body of XLSX export response")?;
  debug!("recieved XLSX sheet export");

  let payload = Cursor::new(body);

  let main_sheet = Xlsx::new(payload.clone())
    .into_diagnostic()
    .context("failed to decode XLSX export values as XLSX")?;
  debug!("decoded XLSX export values");
  let styles_only_sheet =
    umya_spreadsheet::reader::xlsx::read_reader(payload, false)
      .into_diagnostic()
      .context("failed to decode XLSX export styles as XLSX")?;

  Ok(DecodedSpreadsheet {
    main:   main_sheet,
    styles: styles_only_sheet,
  })
}
