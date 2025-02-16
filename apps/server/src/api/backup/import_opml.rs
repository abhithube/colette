use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use colette_core::backup::BackupService;

use super::BACKUPS_TAG;
use crate::api::common::{AuthUser, Error};

#[utoipa::path(
  post,
  path = "/opml/import",
  request_body = Vec<u8>,
  responses(ImportOpmlResponse),
  operation_id = "importOpml",
  description = "Import OPML feeds",
  tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<BackupService>>,
    AuthUser(user_id): AuthUser,
    bytes: Bytes,
) -> Result<ImportOpmlResponse, Error> {
    match service.import_opml(bytes, user_id).await {
        Ok(_) => Ok(ImportOpmlResponse::NoContent),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ImportOpmlResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportOpmlResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
