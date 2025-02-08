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

use super::{Bookmark, BookmarkState};
use crate::api::common::{BOOKMARKS_TAG, BaseError, Error, Id, Session};

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BookmarkUpdate {
    #[schema(min_length = 1)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub title: Option<Option<String>>,
    #[schema(value_type = Option<Url>)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub thumbnail_url: Option<Option<Url>>,
    #[schema(value_type = Option<DateTime<Utc>>)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub published_at: Option<Option<DateTime<Utc>>>,
    #[schema(min_length = 1)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub author: Option<Option<String>>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub folder_id: Option<Option<Uuid>>,
    #[schema(nullable = false, min_length = 1)]
    pub tags: Option<Vec<String>>,
}

impl From<BookmarkUpdate> for bookmark::BookmarkUpdate {
    fn from(value: BookmarkUpdate) -> Self {
        Self {
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
    State(state): State<BookmarkState>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<BookmarkUpdate>,
) -> Result<UpdateResponse, Error> {
    match state
        .service
        .update_bookmark(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(Box::new(
            (data, state.bucket_url.clone()).into(),
        ))),
        Err(e) => match e {
            bookmark::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code)]
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
