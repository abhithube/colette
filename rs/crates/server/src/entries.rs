use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::entries::{self, EntriesService, ListEntriesParams, UpdateEntry};
use uuid::Uuid;

use crate::common::{BaseError, Context, EntryList, Error, Id, Paginated, Session};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_entries, mark_entry_as_read, mark_entry_as_unread),
    components(schemas(Entry))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/entries",
            Router::new()
                .route("/", routing::get(list_entries))
                .route("/:id/markAsRead", routing::get(mark_entry_as_read))
                .route("/:id/markAsUnread", routing::get(mark_entry_as_unread)),
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

#[utoipa::path(
    post,
    path = "/{id}/markAsRead",
    params(Id),
    responses(UpdateResponse),
    operation_id = "markEntryAsRead",
    description = "Mark a feed entry as read",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn mark_entry_as_read(
    State(service): State<Arc<EntriesService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    mark_entry(id, true, session, service).await
}

#[utoipa::path(
    post,
    path = "/{id}/markAsUnread",
    params(Id),
    responses(UpdateResponse),
    operation_id = "markEntryAsUnread",
    description = "Mark a feed entry as unread",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn mark_entry_as_unread(
    State(service): State<Arc<EntriesService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    mark_entry(id, false, session, service).await
}

async fn mark_entry(
    id: Uuid,
    has_read: bool,
    session: Session,
    service: Arc<EntriesService>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .update(
            id,
            UpdateEntry {
                has_read: Some(has_read),
            },
            session.into(),
        )
        .await
        .map(Entry::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            entries::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub description: Option<String>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(format = "uri", required)]
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
    #[param(nullable = false)]
    pub published_at: Option<DateTime<Utc>>,
    #[param(nullable = false)]
    pub feed_id: Option<Uuid>,
    #[param(nullable = false)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated entry")]
    Ok(Entry),

    #[response(status = 404, description = "Entry not found")]
    NotFound(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
