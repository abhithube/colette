use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::entries::{EntriesService, ListEntriesParams};
use uuid::Uuid;

use crate::common::{Context, EntryList, Error, Paginated, Session};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_entries), components(schemas(Entry)))]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/entries",
            Router::new().route("/", routing::get(list_entries)),
        )
    }
}

#[utoipa::path(
    get,
    path = "",
    params(ListEntriesQuery),
    responses(ListResponse),
    operation_id = "listEntries",
    description = "List feed entries",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn list_entries(
    State(service): State<Arc<EntriesService>>,
    Valid(Query(query)): Valid<Query<ListEntriesQuery>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(query.into(), session.into())
        .await
        .map(Paginated::<Entry>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    #[schema(format = "uri")]
    pub thumbnail_url: Option<String>,
    pub has_read: bool,
    pub feed_id: Uuid,
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

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams, validator::Validate)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListEntriesQuery {
    pub published_at: Option<DateTime<Utc>>,
    pub feed_id: Option<Uuid>,
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
