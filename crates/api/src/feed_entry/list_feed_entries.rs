use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    feed_entry::{FeedEntryCursor, ListFeedEntriesQuery},
};
use uuid::Uuid;

use crate::{
    ApiState,
    common::{ApiError, Query},
    feed_entry::{FEED_ENTRIES_TAG, FeedEntry},
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
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<FeedEntryCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_feed_entries
        .handle(ListFeedEntriesQuery {
            feed_id: query.feed_id.map(Into::into),
            cursor,
            limit: Some(PAGINATION_LIMIT),
        })
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
