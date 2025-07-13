use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed_entry::{self, FeedEntryCursor};
use uuid::Uuid;

use super::{FEED_ENTRIES_TAG, FeedEntry};
use crate::{
    ApiState,
    common::{ApiError, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(FeedEntryListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listFeedEntries",
    description = "List feed entries",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<FeedEntryListQuery>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .feed_entry_service
        .list_feed_entries(query.try_into()?)
        .await
    {
        Ok(feed_entries) => {
            let data = feed_entries
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct FeedEntryListQuery {
    /// Filter by the ID of the associated RSS feed
    #[param(nullable = false)]
    feed_id: Option<Uuid>,
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

impl TryFrom<FeedEntryListQuery> for feed_entry::FeedEntryListQuery {
    type Error = ErrResponse;

    fn try_from(value: FeedEntryListQuery) -> Result<Self, Self::Error> {
        let cursor = value
            .cursor
            .map(|e| decode_cursor::<FeedEntryCursor>(&e))
            .transpose()
            .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

        Ok(Self {
            feed_id: value.feed_id,
            cursor,
            limit: Some(PAGINATION_LIMIT),
        })
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of feed entries")]
pub(super) struct OkResponse(Paginated<FeedEntry>);

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
