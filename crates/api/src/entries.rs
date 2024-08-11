use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::{
    common::PAGINATION_LIMIT,
    entries::{self, EntriesFindManyFilters, EntriesRepository, EntriesUpdateData},
};
use uuid::Uuid;

use crate::common::{BaseError, EntryList, Error, Id, Paginated, Session};

#[derive(Clone, axum::extract::FromRef)]
pub struct EntriesState {
    pub repository: Arc<dyn EntriesRepository>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_entries, get_entry, update_entry),
    components(schemas(Entry, EntryUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<EntriesState> {
        Router::new().nest(
            "/entries",
            Router::new()
                .route("/", routing::get(list_entries))
                .route("/:id", routing::get(get_entry).patch(update_entry)),
        )
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
    State(repository): State<Arc<dyn EntriesRepository>>,
    Query(query): Query<ListEntriesQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_many_entries(
            session.profile_id,
            Some((PAGINATION_LIMIT + 1) as u64),
            query.cursor,
            Some(EntriesFindManyFilters {
                feed_id: query.feed_id,
                has_read: query.has_read,
                tags: query.tags,
            }),
        )
        .await
        .map(Paginated::<Entry>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListEntriesQuery {
    #[param(nullable = false)]
    pub feed_id: Option<Uuid>,
    #[param(nullable = false)]
    pub has_read: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
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

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getEntry",
    description = "Get a feed entry by ID",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn get_entry(
    State(repository): State<Arc<dyn EntriesRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_one_entry(id, session.profile_id)
        .await
        .map(Entry::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            entries::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Entry by ID")]
    Ok(Entry),

    #[response(status = 404, description = "Entry not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = EntryUpdate,
    responses(UpdateResponse),
    operation_id = "updateEntry",
    description = "Update a feed entry by ID",
    tag = "Entries"
)]
#[axum::debug_handler]
pub async fn update_entry(
    State(repository): State<Arc<dyn EntriesRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<EntryUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update_entry(id, session.profile_id, body.into())
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

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct EntryUpdate {
    pub has_read: Option<bool>,
}

impl From<EntryUpdate> for EntriesUpdateData {
    fn from(value: EntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated entry")]
    Ok(Entry),

    #[response(status = 404, description = "Entry not found")]
    NotFound(BaseError),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
