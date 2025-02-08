use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark;

use super::{Bookmark, BookmarkState};
use crate::api::common::{BOOKMARKS_TAG, BaseError, Error, Id, Session};

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
pub async fn handler(
    State(state): State<BookmarkState>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match state.service.get_bookmark(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok((data, state.bucket_url.clone()).into())),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(clippy::large_enum_variant)]
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
