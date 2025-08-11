use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use colette_core::{Handler as _, subscription::ExportSubscriptionsQuery};

use crate::{
    ApiState,
    common::{ApiError, Auth},
    subscription::SUBSCRIPTIONS_TAG,
};

#[utoipa::path(
  post,
  path = "/export",
  responses(OkResponse, ErrResponse),
  operation_id = "exportSubscriptions",
  description = "Export user subscriptions",
  tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .export_subscriptions
        .handle(ExportSubscriptionsQuery { user_id })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(utoipa::IntoResponses)]
#[response(
    status = 200,
    description = "OPML subscriptions file",
    content_type = "application/xml"
)]
pub(super) struct OkResponse(Vec<u8>);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", HeaderValue::from_static("application/xml"));

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
