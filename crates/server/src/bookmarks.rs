use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::bookmarks::{
    self, BookmarksService, CreateBookmark, ListBookmarksParams, UpdateBookmark,
};
use uuid::Uuid;

use crate::common::{BaseError, BookmarkList, Context, Error, Id, Paginated, Session};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_bookmarks, create_bookmark, update_bookmark, delete_bookmark),
    components(schemas(Bookmark, BookmarkCreate, BookmarkUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/bookmarks",
            Router::new()
                .route("/", routing::get(list_bookmarks).post(create_bookmark))
                .route(
                    "/:id",
                    routing::patch(update_bookmark).delete(delete_bookmark),
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
    pub title: String,
    #[schema(format = "uri", required)]
    pub thumbnail_url: Option<String>,
    #[schema(required)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub author: Option<String>,
    #[schema(required)]
    pub custom_title: Option<String>,
    #[schema(format = "uri", required)]
    pub custom_thumbnail_url: Option<String>,
    #[schema(required)]
    pub custom_published_at: Option<DateTime<Utc>>,
    #[schema(required)]
    pub custom_author: Option<String>,
    #[schema(required)]
    pub collection_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
            custom_title: value.custom_title,
            custom_thumbnail_url: value.custom_thumbnail_url,
            custom_published_at: value.custom_published_at,
            custom_author: value.custom_author,
            collection_id: value.collection_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(ListBookmarksQuery),
    responses(ListResponse),
    operation_id = "listBookmarks",
    description = "List the active profile bookmarks",
    tag = "Bookmarks"
)]
#[axum::debug_handler]
pub async fn list_bookmarks(
    State(service): State<Arc<BookmarksService>>,
    Valid(Query(query)): Valid<Query<ListBookmarksQuery>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(query.into(), session.into())
        .await
        .map(Paginated::<Bookmark>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams, validator::Validate)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListBookmarksQuery {
    #[param(nullable = false)]
    pub published_at: Option<DateTime<Utc>>,
    #[param(nullable = false)]
    pub collection_id: Option<Uuid>,
    #[param(nullable = false)]
    pub is_default: Option<bool>,
}

impl From<ListBookmarksQuery> for ListBookmarksParams {
    fn from(value: ListBookmarksQuery) -> Self {
        Self {
            published_at: value.published_at,
            collection_id: value.collection_id,
            is_default: value.is_default,
        }
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

#[utoipa::path(
    post,
    path = "",
    request_body = BookmarkCreate,
    responses(CreateResponse),
    operation_id = "createBookmark",
    description = "Add a bookmark to a collection",
    tag = "Bookmarks"
  )]
#[axum::debug_handler]
pub async fn create_bookmark(
    State(service): State<Arc<BookmarksService>>,
    session: Session,
    Valid(Json(body)): Valid<Json<BookmarkCreate>>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .create(body.into(), session.into())
        .await
        .map(Bookmark::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(Box::new(data))),
        Err(e) => match e {
            bookmarks::Error::Scraper(_) => Ok(CreateResponse::BadGateway(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkCreate {
    #[schema(format = "uri")]
    #[validate(url(message = "not a valid URL"))]
    pub url: String,
    #[schema(nullable = false)]
    pub collection_id: Option<Uuid>,
}

impl From<BookmarkCreate> for CreateBookmark {
    fn from(value: BookmarkCreate) -> Self {
        Self {
            url: value.url,
            collection_id: value.collection_id,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created bookmark")]
    Created(Box<Bookmark>),

    #[allow(dead_code)]
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

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = BookmarkUpdate,
    responses(UpdateResponse),
    operation_id = "updateBookmark",
    description = "Update a bookmark by ID",
    tag = "Bookmarks"
)]
#[axum::debug_handler]
pub async fn update_bookmark(
    State(service): State<Arc<BookmarksService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<BookmarkUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .update(id, body.into(), session.into())
        .await
        .map(Bookmark::from);

    match result {
        Ok(bookmark) => Ok(UpdateResponse::Ok(Box::new(bookmark))),
        Err(e) => match e {
            bookmarks::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkUpdate {
    #[schema(min_length = 1, nullable = false)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: Option<String>,
    #[schema(format = "uri", nullable = false)]
    #[validate(url(message = "not a valid URL"))]
    pub thumbnail_url: Option<String>,
    #[schema(nullable = false)]
    pub published_at: Option<DateTime<Utc>>,
    #[schema(min_length = 1, nullable = false)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub author: Option<String>,
}

impl From<BookmarkUpdate> for UpdateBookmark {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated bookmark")]
    Ok(Box<Bookmark>),

    #[response(status = 404, description = "Bookmark not found")]
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

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteBookmark",
    description = "Delete a bookmark by ID",
    tag = "Bookmarks"
)]
#[axum::debug_handler]
pub async fn delete_bookmark(
    State(service): State<Arc<BookmarksService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.delete(id, session.into()).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            bookmarks::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
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
