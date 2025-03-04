use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;
use uuid::Uuid;

use super::{BOOKMARKS_TAG, Bookmark};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
};

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
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<BookmarkUpdate>,
) -> Result<UpdateResponse, Error> {
    match state
        .bookmark_service
        .update_bookmark(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(
            (data, state.image_base_url.clone()).into(),
        )),
        Err(e) => match e {
            bookmark::Error::Forbidden(_) => Ok(UpdateResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            bookmark::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    #[schema(value_type = Option<Url>)]
    pub thumbnail_url: Option<Option<Url>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    #[schema(value_type = Option<DateTime<Utc>>)]
    pub published_at: Option<Option<DateTime<Utc>>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    #[schema(value_type = Option<Option<String>>, min_length = 1)]
    pub author: Option<Option<NonEmptyString>>,
    #[schema(nullable = false)]
    pub tags: Option<Vec<Uuid>>,
}

impl From<BookmarkUpdate> for bookmark::BookmarkUpdate {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author.map(|e| e.map(Into::into)),
            tags: value.tags,
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated bookmark")]
    Ok(Bookmark),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Bookmark not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
