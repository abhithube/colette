use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

use super::BOOKMARKS_TAG;
use crate::{
    ApiState,
    common::{ApiError, Auth},
};

#[utoipa::path(
  post,
  path = "/export",
  responses(OkResponse, ErrResponse),
  operation_id = "exportBookmarks",
  description = "Export user bookmarks",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state.bookmark_service.export_bookmarks(user_id).await {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(
    status = 200,
    description = "Netscape bookmarks file",
    content_type = "text/html"
)]
pub(super) struct OkResponse(Vec<u8>);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("text/html"));

        (headers, self.0).into_response()
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
