use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::bookmark;

use super::{BOOKMARKS_TAG, BookmarkDetails};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
  get,
  path = "/{id}",
  params(Id, BookmarkGetQuery),
  responses(GetResponse),
  operation_id = "getBookmark",
  description = "Get a bookmark by ID",
  tag = BOOKMARKS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<BookmarkGetQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<GetResponse, Error> {
    match state
        .bookmark_service
        .get_bookmark(
            bookmark::BookmarkGetQuery {
                id,
                with_tags: query.with_tags,
            },
            user_id,
        )
        .await
    {
        Ok(data) => Ok(GetResponse::Ok((data, state.image_base_url.clone()).into())),
        Err(e) => match e {
            bookmark::Error::Forbidden(_) => Ok(GetResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            bookmark::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct BookmarkGetQuery {
    #[serde(default = "with_tags")]
    pub with_tags: bool,
}

fn with_tags() -> bool {
    false
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Bookmark by ID")]
    Ok(BookmarkDetails),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Bookmark not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
