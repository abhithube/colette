use axum::{
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use colette_core::entries::ListEntriesParams;

use crate::common::EntryList;

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    #[schema(format = "uri")]
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: String,
}

impl From<colette_core::Entry> for Entry {
    fn from(value: colette_core::Entry) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.feed_id,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams, validator::Validate)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListEntriesQuery {
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<String>,
    pub has_read: Option<bool>,
}

impl From<ListEntriesQuery> for ListEntriesParams {
    fn from(value: ListEntriesQuery) -> Self {
        Self {
            published_at: value.published_at,
            feed_id: value.feed_id,
            has_read: value.has_read,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of entries")]
    Ok(EntryList),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
