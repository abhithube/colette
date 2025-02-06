use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{self, BookmarkService},
    common::NonEmptyString,
};
use url::Url;
use uuid::Uuid;

use super::Bookmark;
use crate::api::common::{BOOKMARKS_TAG, BaseError, Error, Session};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
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

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
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
