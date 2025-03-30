use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;

use super::BOOKMARKS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, Error},
};

#[utoipa::path(
  post,
  path = "/import",
  request_body = Vec<u8>,
  responses(ImportBookmarksResponse),
  operation_id = "importBookmarks",
  description = "Import bookmarks into user account",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    bytes: Bytes,
) -> Result<ImportBookmarksResponse, Error> {
    match state
        .bookmark_service
        .import_bookmarks(bytes, user_id)
        .await
    {
        Ok(_) => Ok(ImportBookmarksResponse::NoContent),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ImportBookmarksResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportBookmarksResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
