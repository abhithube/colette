use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed_entry;

use super::{FEED_ENTRIES_TAG, FeedEntry};
use crate::api::{
    ApiState,
    common::{BaseError, Error, Id},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeedEntry",
    description = "Get a feed entry by ID",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
) -> Result<GetResponse, Error> {
    match state.feed_entry_service.get_feed_entry(id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            feed_entry::Error::Forbidden(_) => Ok(GetResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            feed_entry::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Feed entry by ID")]
    Ok(FeedEntry),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Feed entry not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
