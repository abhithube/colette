use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark::{self, BookmarkService};

use crate::{
    Session,
    common::{BOOKMARKS_TAG, BaseError, Error, Id},
};

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
pub async fn handler(
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
