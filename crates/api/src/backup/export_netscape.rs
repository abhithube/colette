use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};
use colette_core::backup::BackupService;

use crate::common::{Error, Session, BACKUPS_TAG};

#[utoipa::path(
  post,
  path = "/netscape/export",
  responses(ExportNetscapeResponse),
  operation_id = "exportNetscape",
  description = "Export Netscape bookmarks",
  tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<BackupService>>,
    session: Session,
) -> Result<ExportNetscapeResponse, Error> {
    match service.export_netscape(session.user_id).await {
        Ok(data) => Ok(ExportNetscapeResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ExportNetscapeResponse {
    #[response(
        status = 200,
        description = "Netscape file",
        content_type = "text/html"
    )]
    Ok(Vec<u8>),
}

impl IntoResponse for ExportNetscapeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => {
                let mut headers = HeaderMap::new();
                headers.insert("Content-Type", HeaderValue::from_static("text/html"));

                (headers, data).into_response()
            }
        }
    }
}
