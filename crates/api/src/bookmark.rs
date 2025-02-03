use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{self, BookmarkService},
    common::NonEmptyString,
};
use url::Url;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Id, Session, BOOKMARKS_TAG},
    tag::Tag,
    Paginated,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct BookmarkState {
    service: Arc<BookmarkService>,
}

impl BookmarkState {
    pub fn new(service: Arc<BookmarkService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Bookmark, Paginated<Bookmark>, BookmarkCreate, BookmarkUpdate, BookmarkScrape, BookmarkScraped)))]
pub struct BookmarkApi;

impl BookmarkApi {
    pub fn router() -> OpenApiRouter<BookmarkState> {
        OpenApiRouter::with_openapi(BookmarkApi::openapi())
            .routes(routes!(list_bookmarks, create_bookmark))
            .routes(routes!(get_bookmark, update_bookmark, delete_bookmark))
            .routes(routes!(scrape_bookmark))
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri", required)]
    pub thumbnail_url: Option<String>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(required)]
    pub folder_id: Option<Uuid>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<Tag>>,
}

impl From<colette_core::Bookmark> for Bookmark {
    fn from(value: colette_core::Bookmark) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkCreate {
    pub url: Url,
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    #[schema(value_type = Option<String>, min_length = 1)]
    pub author: Option<NonEmptyString>,
    pub folder_id: Option<Uuid>,
    #[schema(value_type = Option<Vec<String>>, nullable = false, min_length = 1)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<BookmarkCreate> for bookmark::BookmarkCreate {
    fn from(value: BookmarkCreate) -> Self {
        Self {
            url: value.url,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            folder_id: value.folder_id,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkUpdate {
    #[schema(value_type = Option<String>, min_length = 1)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub title: Option<Option<NonEmptyString>>,
    #[schema(value_type = Option<Url>)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub thumbnail_url: Option<Option<Url>>,
    #[schema(value_type = Option<DateTime<Utc>>)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub published_at: Option<Option<DateTime<Utc>>>,
    #[schema(value_type = Option<String>, min_length = 1)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub author: Option<Option<NonEmptyString>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub folder_id: Option<Option<Uuid>>,
    #[schema(value_type = Option<Vec<String>>, nullable = false, min_length = 1)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<BookmarkUpdate> for bookmark::BookmarkUpdate {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            folder_id: value.folder_id,
            tags: value.tags,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct BookmarkListQuery {
    #[param(nullable = false)]
    pub filter_by_folder: Option<bool>,
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<NonEmptyString>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<BookmarkListQuery> for bookmark::BookmarkListQuery {
    fn from(value: BookmarkListQuery) -> Self {
        Self {
            folder_id: if value.filter_by_folder.unwrap_or(value.folder_id.is_some()) {
                Some(value.folder_id)
            } else {
                None
            },
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
            cursor: value.cursor,
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkScrape {
    #[schema(format = "uri")]
    pub url: Url,
}

impl From<BookmarkScrape> for bookmark::BookmarkScrape {
    fn from(value: BookmarkScrape) -> Self {
        Self { url: value.url }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkScraped {
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri", required)]
    pub thumbnail_url: Option<String>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
}

impl From<bookmark::BookmarkScraped> for BookmarkScraped {
    fn from(value: bookmark::BookmarkScraped) -> Self {
        Self {
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(BookmarkListQuery),
    responses(ListResponse),
    operation_id = "listBookmarks",
    description = "List user bookmarks",
    tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn list_bookmarks(
    State(service): State<Arc<BookmarkService>>,
    Query(query): Query<BookmarkListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_bookmarks(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getBookmark",
    description = "Get a bookmark by ID",
    tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn get_bookmark(
    State(service): State<Arc<BookmarkService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_bookmark(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "",
    request_body = BookmarkCreate,
    responses(CreateResponse),
    operation_id = "createBookmark",
    description = "Add a bookmark",
    tag = BOOKMARKS_TAG
  )]
#[axum::debug_handler]
pub async fn create_bookmark(
    State(service): State<Arc<BookmarkService>>,
    session: Session,
    Json(body): Json<BookmarkCreate>,
) -> Result<CreateResponse, Error> {
    match service.create_bookmark(body.into(), session.user_id).await {
        Ok(data) => Ok(CreateResponse::Created(Box::new(data.into()))),
        Err(e) => match e {
            bookmark::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = BookmarkUpdate,
    responses(UpdateResponse),
    operation_id = "updateBookmark",
    description = "Update a bookmark by ID",
    tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn update_bookmark(
    State(service): State<Arc<BookmarkService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<BookmarkUpdate>,
) -> Result<UpdateResponse, Error> {
    match service
        .update_bookmark(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(Box::new(data.into()))),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteBookmark",
    description = "Delete a bookmark by ID",
    tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn delete_bookmark(
    State(service): State<Arc<BookmarkService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_bookmark(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "/scrape",
    request_body = BookmarkScrape,
    responses(ScrapeResponse),
    operation_id = "scrapeBookmark",
    description = "Scrape bookmark from a webpage",
    tag = BOOKMARKS_TAG
  )]
#[axum::debug_handler]
pub async fn scrape_bookmark(
    State(service): State<Arc<BookmarkService>>,
    Json(body): Json<BookmarkScrape>,
) -> Result<ScrapeResponse, Error> {
    match service.scrape_bookmark(body.into()).await {
        Ok(data) => Ok(ScrapeResponse::Ok(data.into())),
        Err(bookmark::Error::Scraper(e)) => Ok(ScrapeResponse::BadGateway(BaseError {
            message: e.to_string(),
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of bookmarks")]
    Ok(Paginated<Bookmark>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Bookmark by ID")]
    Ok(Bookmark),

    #[response(status = 404, description = "Bookmark not found")]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created bookmark")]
    Created(Box<Bookmark>),

    #[response(status = 409, description = "Bookmark already exists")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(data) => (StatusCode::CONFLICT, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated bookmark")]
    Ok(Box<Bookmark>),

    #[response(status = 404, description = "Bookmark not found")]
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

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted bookmark")]
    NoContent,

    #[response(status = 404, description = "Bookmark not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ScrapeResponse {
    #[response(status = 201, description = "Scraped bookmark")]
    Ok(BookmarkScraped),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse bookmark")]
    BadGateway(BaseError),
}

impl IntoResponse for ScrapeResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
    }
}
