use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed_entry;

use super::{FEED_ENTRIES_TAG, FeedEntry};
use crate::{
    ApiState,
    common::{ApiError, Id, Path},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(OkResponse, ErrResponse),
    operation_id = "getFeedEntry",
    description = "Get a feed entry by ID",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
) -> Result<OkResponse, ErrResponse> {
    match state.feed_entry_service.get_feed_entry(id).await {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            feed_entry::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            feed_entry::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Feed entry by ID")]
pub(super) struct OkResponse(FeedEntry);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Feed entry not found")]
    NotFound(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
