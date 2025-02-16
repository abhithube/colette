use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::bookmark;
use url::Url;
use uuid::Uuid;

use super::{BOOKMARKS_TAG, Bookmark, BookmarkState};
use crate::api::common::{AuthUser, BaseError, Error, NonEmptyString};

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
pub async fn handler(
    State(state): State<BookmarkState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<BookmarkCreate>,
) -> Result<CreateResponse, Error> {
    match state.service.create_bookmark(body.into(), user_id).await {
        Ok(data) => Ok(CreateResponse::Created(
            (data, state.bucket_url.clone()).into(),
        )),
        Err(e) => match e {
            bookmark::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkCreate {
    pub url: Url,
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub thumbnail_url: Option<Url>,
    pub published_at: Option<DateTime<Utc>>,
    #[schema(value_type = String, min_length = 1)]
    pub author: Option<NonEmptyString>,
    pub folder_id: Option<Uuid>,
    #[schema(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<BookmarkCreate> for bookmark::BookmarkCreate {
    fn from(value: BookmarkCreate) -> Self {
        Self {
            url: value.url,
            title: value.title.into(),
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author.map(Into::into),
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created bookmark")]
    Created(Bookmark),

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
