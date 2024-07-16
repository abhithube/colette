use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::bookmarks::{self, BookmarksService, ListBookmarksParams};
use uuid::Uuid;

use crate::common::{BaseError, BookmarkList, Context, Error, Id, Paginated, Session};

#[derive(utoipa::OpenApi)]
#[openapi(paths(list_bookmarks, delete_bookmark), components(schemas(Bookmark)))]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/bookmarks",
            Router::new()
                .route("/", routing::get(list_bookmarks))
                .route("/:id", routing::delete(delete_bookmark)),
        )
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

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub id: Uuid,
    #[schema(format = "uri")]
    pub link: String,
    pub title: String,
    #[schema(format = "uri")]
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub custom_title: Option<String>,
    #[schema(format = "uri")]
    pub custom_thumbnail_url: Option<String>,
    pub custom_published_at: Option<DateTime<Utc>>,
    pub custom_author: Option<String>,
    pub collection_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, serde::Deserialize, utoipa::IntoParams, validator::Validate)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListBookmarksQuery {
    pub published_at: Option<DateTime<Utc>>,
    pub collection_id: Option<Uuid>,
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