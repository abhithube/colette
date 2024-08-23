use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_extra::extract::Query;
use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{
        self, BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        ProcessedBookmark,
    },
    common::{IdParams, PAGINATION_LIMIT},
    scraper::Scraper,
};
use url::Url;
use uuid::Uuid;

use crate::{
    common::{BaseError, BookmarkList, Error, Id, Paginated, Session},
    tag::{Tag, TagCreate},
};

#[derive(Clone, axum::extract::FromRef)]
pub struct BookmarkState {
    repository: Arc<dyn BookmarkRepository>,
    scraper: Arc<dyn Scraper<ProcessedBookmark>>,
}

impl BookmarkState {
    pub fn new(
        repository: Arc<dyn BookmarkRepository>,
        scraper: Arc<dyn Scraper<ProcessedBookmark>>,
    ) -> Self {
        Self {
            repository,
            scraper,
        }
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_bookmarks,
        get_bookmark,
        create_bookmark,
        update_bookmark,
        delete_bookmark
    ),
    components(schemas(Bookmark, BookmarkList, BookmarkCreate, BookmarkUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<BookmarkState> {
        Router::new().nest(
            "/bookmarks",
            Router::new()
                .route("/", routing::get(list_bookmarks).post(create_bookmark))
                .route(
                    "/:id",
                    routing::get(get_bookmark)
                        .patch(update_bookmark)
                        .delete(delete_bookmark),
                ),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    #[schema(required)]
    pub title: String,
    #[schema(format = "uri", required)]
    pub thumbnail_url: Option<String>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(required)]
    pub collection_id: Option<Uuid>,
    pub sort_index: u32,
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
            sort_index: value.sort_index,
            collection_id: value.collection_id,
            tags: value.tags.map(|e| e.into_iter().map(Tag::from).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkCreate {
    #[schema(format = "uri")]
    pub url: Url,
    pub collection_id: Option<Uuid>,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkUpdate {
    #[schema(nullable = false)]
    pub sort_index: Option<u32>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub collection_id: Option<Option<Uuid>>,
    #[schema(nullable = false)]
    pub tags: Option<Vec<TagCreate>>,
}

impl From<BookmarkUpdate> for BookmarkUpdateData {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            sort_index: value.sort_index,
            collection_id: value.collection_id,
            tags: value
                .tags
                .map(|e| e.into_iter().map(|e| e.title.into()).collect()),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListBookmarksQuery {
    #[param(nullable = false)]
    pub filter_by_collection: Option<bool>,
    #[param(nullable = false)]
    pub collection_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<ListBookmarksQuery> for BookmarkFindManyFilters {
    fn from(value: ListBookmarksQuery) -> Self {
        Self {
            collection_id: if value
                .filter_by_collection
                .unwrap_or(value.collection_id.is_some())
            {
                Some(value.collection_id)
            } else {
                None
            },
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(ListBookmarksQuery),
    responses(ListResponse),
    operation_id = "listBookmarks",
    description = "List the active profile bookmarks"
)]
#[axum::debug_handler]
pub async fn list_bookmarks(
    State(repository): State<Arc<dyn BookmarkRepository>>,
    Query(query): Query<ListBookmarksQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .list(
            session.profile_id,
            Some(PAGINATION_LIMIT),
            query.cursor.clone(),
            Some(query.into()),
        )
        .await
        .map(Paginated::<Bookmark>::from);

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
    operation_id = "getBookmark",
    description = "Get a bookmark by ID"
)]
#[axum::debug_handler]
pub async fn get_bookmark(
    State(repository): State<Arc<dyn BookmarkRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find(IdParams::new(id, session.profile_id))
        .await
        .map(Bookmark::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    post,
    path = "",
    request_body = BookmarkCreate,
    responses(CreateResponse),
    operation_id = "createBookmark",
    description = "Add a bookmark to a profile",
  )]
#[axum::debug_handler]
pub async fn create_bookmark(
    State(BookmarkState {
        repository,
        scraper,
    }): State<BookmarkState>,
    session: Session,
    Json(mut body): Json<BookmarkCreate>,
) -> Result<impl IntoResponse, Error> {
    let scraped = scraper.scrape(&mut body.url);
    if let Err(e) = scraped {
        return Ok(CreateResponse::BadGateway(BaseError {
            message: e.to_string(),
        }));
    }

    let result = repository
        .create(BookmarkCreateData {
            url: body.url.into(),
            bookmark: scraped.unwrap(),
            collection_id: body.collection_id,
            profile_id: session.profile_id,
        })
        .await
        .map(Bookmark::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(Box::new(data))),
        _ => Err(Error::Unknown),
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
)]
#[axum::debug_handler]
pub async fn update_bookmark(
    State(repository): State<Arc<dyn BookmarkRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<BookmarkUpdate>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update(IdParams::new(id, session.profile_id), body.into())
        .await
        .map(Bookmark::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(Box::new(data))),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteBookmark",
    description = "Delete a bookmark by ID"
)]
#[axum::debug_handler]
pub async fn delete_bookmark(
    State(repository): State<Arc<dyn BookmarkRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .delete(IdParams::new(id, session.profile_id))
        .await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of bookmarks")]
    Ok(BookmarkList),
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created bookmark")]
    Created(Box<Bookmark>),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),

    #[response(status = 502, description = "Failed to fetch or parse bookmark")]
    BadGateway(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
            Self::BadGateway(e) => (StatusCode::BAD_GATEWAY, e).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[derive(Debug, utoipa::IntoResponses)]
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
