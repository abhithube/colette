use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::{
    common::NonEmptyString,
    feed_entry::{self, FeedEntryService},
};
use uuid::Uuid;

use super::FeedEntry;
use crate::api::common::{Error, FEED_ENTRIES_TAG, Paginated, Session};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FeedEntryListQuery {
    #[param(nullable = false)]
    pub feed_id: Option<Uuid>,
    #[param(nullable = false)]
    pub smart_feed_id: Option<Uuid>,
    #[param(nullable = false)]
    pub has_read: Option<bool>,
    #[param(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<NonEmptyString>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<FeedEntryListQuery> for feed_entry::FeedEntryListQuery {
    fn from(value: FeedEntryListQuery) -> Self {
        Self {
            feed_id: value.feed_id,
            smart_feed_id: value.smart_feed_id,
            has_read: value.has_read,
            tags: value.tags,
            cursor: value.cursor,
        }
    }
}

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
    State(service): State<Arc<FeedEntryService>>,
    Query(query): Query<FeedEntryListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service
        .list_feed_entries(query.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
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
