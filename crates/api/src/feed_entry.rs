use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use colette_core::feed_entry::{self, FeedEntryService};
use http::StatusCode;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Id, Session, FEED_ENTRIES_TAG},
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct FeedEntryState {
    service: Arc<FeedEntryService>,
}

impl FeedEntryState {
    pub fn new(service: Arc<FeedEntryService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(FeedEntry, Paginated<FeedEntry>, FeedEntryUpdate)))]
pub struct FeedEntryApi;

impl FeedEntryApi {
    pub fn router() -> OpenApiRouter<FeedEntryState> {
        OpenApiRouter::with_openapi(FeedEntryApi::openapi())
            .routes(routes!(list_feed_entries))
            .routes(routes!(get_feed_entry, update_feed_entry))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntry {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
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

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedEntryUpdate {
    pub has_read: Option<bool>,
}

impl From<FeedEntryUpdate> for feed_entry::FeedEntryUpdate {
    fn from(value: FeedEntryUpdate) -> Self {
        Self {
            has_read: value.has_read,
        }
    }
}

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
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
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
pub async fn list_feed_entries(
    State(service): State<Arc<FeedEntryService>>,
    Query(query): Query<FeedEntryListQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service
        .list_feed_entries(query.into(), session.profile_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        _ => Err(Error::Unknown),
    }
}

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
pub async fn get_feed_entry(
    State(service): State<Arc<FeedEntryService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_feed_entry(id, session.profile_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
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
    description = "Update a feed entry by ID",
    tag = FEED_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn update_feed_entry(
    State(service): State<Arc<FeedEntryService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FeedEntryUpdate>,
) -> Result<impl IntoResponse, Error> {
    match service
        .update_feed_entry(id, body.into(), session.profile_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
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
    Ok(Paginated<FeedEntry>),
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
