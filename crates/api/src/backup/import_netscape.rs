use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use colette_task::import_bookmarks;

use super::BackupState;
use crate::common::{Error, Session, BACKUPS_TAG};

#[utoipa::path(
  post,
  path = "/netscape/import",
  request_body = Vec<u8>,
  responses(ImportResponse),
  operation_id = "importNetscape",
  description = "Import Netscape bookmarks",
  tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<BackupState>,
    session: Session,
    bytes: Bytes,
) -> Result<ImportResponse, Error> {
    match state
        .backup_service
        .import_netscape(bytes, session.user_id)
        .await
    {
        Ok(urls) => {
            let mut storage = state.import_bookmarks_storage.lock().await;

            storage
                .push(import_bookmarks::Job {
                    urls,
                    user_id: session.user_id,
                })
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            Ok(ImportResponse::NoContent)
        }
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ImportResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
