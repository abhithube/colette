use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue},
    response::{IntoResponse, Response},
};

use super::BOOKMARKS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, Error},
};

#[utoipa::path(
  post,
  path = "/export",
  responses(ExportBookmarksResponse),
  operation_id = "exportBookmarks",
  description = "Export user bookmarks",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
) -> Result<ExportBookmarksResponse, Error> {
    match state.bookmark_service.export_bookmarks(user_id).await {
        Ok(data) => Ok(ExportBookmarksResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ExportBookmarksResponse {
    #[response(
        status = 200,
        description = "Bookmarks file",
        content_type = "text/html"
    )]
    Ok(Vec<u8>),
}

impl IntoResponse for ExportBookmarksResponse {
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
