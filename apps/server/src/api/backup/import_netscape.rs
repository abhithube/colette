use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;

use super::BACKUPS_TAG;
use crate::api::{
    ApiState,
    common::{AuthUser, Error},
};

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
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    bytes: Bytes,
) -> Result<ImportResponse, Error> {
    match state.backup_service.import_netscape(bytes, user_id).await {
        Ok(_) => Ok(ImportResponse::NoContent),
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
