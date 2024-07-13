use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use axum_valid::Valid;
use colette_core::bookmarks::{self, BookmarksService};

use super::{
    model::{DeleteResponse, ListBookmarksQuery, ListResponse},
    Bookmark,
};
use crate::{
    common::{self, Id, Paginated},
    error::Error,
    session::Session,
};

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
            bookmarks::Error::NotFound(_) => Ok(DeleteResponse::NotFound(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}
