use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::feed_entry;
use uuid::Uuid;

use super::{FEED_ENTRIES_TAG, FeedEntry};
use crate::{
    ApiState,
    common::{Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(FeedEntryListQuery),
    responses(ListResponse),
    operation_id = "listFeedEntries",
    description = "List feed entries",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<FeedEntryListQuery>,
) -> Result<ListResponse, Error> {
    match state
        .feed_entry_service
        .list_feed_entries(query.into())
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FeedEntryListQuery {
    #[param(nullable = false)]
    pub stream_id: Option<Uuid>,
    #[param(nullable = false)]
    pub feed_id: Option<Uuid>,
    #[param(nullable = false)]
    pub has_read: Option<bool>,
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<Uuid>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<FeedEntryListQuery> for feed_entry::FeedEntryListQuery {
    fn from(value: FeedEntryListQuery) -> Self {
        Self {
            stream_id: value.stream_id,
            feed_id: value.feed_id,
            has_read: value.has_read,
            tags: value.tags,
            cursor: value.cursor,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of feed entries")]
    Ok(Paginated<FeedEntry>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
