use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use colette_task::import_feeds;

use super::BackupState;
use crate::common::{Error, Session, BACKUPS_TAG};

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
    State(state): State<BackupState>,
    session: Session,
    bytes: Bytes,
) -> Result<ImportOpmlResponse, Error> {
    match state
        .backup_service
        .import_opml(bytes, session.user_id)
        .await
    {
        Ok(urls) => {
            let mut storage = state.import_feeds_storage.lock().await;

            storage
                .push(import_feeds::Job { urls })
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            Ok(ImportOpmlResponse::NoContent)
        }
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
