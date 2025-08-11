use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use colette_core::{Handler as _, backup::ImportBackupCommand};

use crate::{
    ApiState,
    backup::BACKUPS_TAG,
    common::{ApiError, Auth},
};

#[utoipa::path(
  post,
  path = "/import",
  request_body = Vec<u8>,
  responses(OkResponse, ErrResponse),
  operation_id = "importBackup",
  description = "Import backup into user account",
  tag = BACKUPS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    raw: Bytes,
) -> Result<OkResponse, ErrResponse> {
    match state
        .import_backup
        .handle(ImportBackupCommand { raw, user_id })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully started import")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
