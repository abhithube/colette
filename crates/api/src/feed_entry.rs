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
    feed_entry::{self, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData},
};
use uuid::Uuid;

use crate::common::{BaseError, Error, FeedEntryList, Id, Paginated, Session};

#[derive(Clone, axum::extract::FromRef)]
pub struct FeedEntryState {
    pub repository: Arc<dyn FeedEntryRepository>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_feed_entries, get_feed_entry, update_feed_entry),
    components(schemas(FeedEntry, FeedEntryList, FeedEntryUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<FeedEntryState> {
        Router::new().nest(
            "/feedEntries",
            Router::new()
                .route("/", routing::get(list_feed_entries))
                .route(
                    "/:id",
                    routing::get(get_feed_entry).patch(update_feed_entry),
                ),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntry {
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

impl From<colette_core::FeedEntry> for FeedEntry {
    fn from(value: colette_core::FeedEntry) -> Self {
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

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntryUpdate {
    pub has_read: Option<bool>,
}

impl From<FeedEntryUpdate> for FeedEntryUpdateData {
    fn from(value: FeedEntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListFeedEntriesQuery {
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

#[utoipa::path(
    get,
    path = "",
    params(ListFeedEntriesQuery),
    responses(ListResponse),
    operation_id = "listFeedEntries",
    description = "List feed entries"
)]
#[axum::debug_handler]
pub async fn list_feed_entries(
    State(repository): State<Arc<dyn FeedEntryRepository>>,
    Query(query): Query<ListFeedEntriesQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_many_feed_entries(
            session.profile_id,
            Some(PAGINATION_LIMIT),
            query.cursor,
            Some(FeedEntryFindManyFilters {
                feed_id: query.feed_id,
                has_read: query.has_read,
                tags: query.tags,
            }),
        )
        .await
        .map(Paginated::<FeedEntry>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeedEntry",
    description = "Get a feed entry by ID"
)]
#[axum::debug_handler]
pub async fn get_feed_entry(
    State(repository): State<Arc<dyn FeedEntryRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_one_feed_entry(id, session.profile_id)
        .await
        .map(FeedEntry::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            feed_entry::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FeedEntryUpdate,
    responses(UpdateResponse),
    operation_id = "updateFeedEntry",
    description = "Update a feed entry by ID"
)]
#[axum::debug_handler]
pub async fn update_feed_entry(
    State(repository): State<Arc<dyn FeedEntryRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<FeedEntryUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update_feed_entry(id, session.profile_id, body.into())
        .await
        .map(FeedEntry::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            feed_entry::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of feed entries")]
    Ok(FeedEntryList),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Feed entry by ID")]
    Ok(FeedEntry),

    #[response(status = 404, description = "Feed entry not found")]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated feed entry")]
    Ok(FeedEntry),

    #[response(status = 404, description = "Feed entry not found")]
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
