use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use colette_core::{bookmark, common::NonEmptyString};
use colette_task::archive_thumbnail;
use url::Url;
use uuid::Uuid;

use super::{Bookmark, BookmarkState};
use crate::{
    common::{BaseError, Error, BOOKMARKS_TAG},
    Session,
};

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
    State(state): State<BookmarkState>,
    session: Session,
    Json(body): Json<BookmarkCreate>,
) -> Result<CreateResponse, Error> {
    match state
        .service
        .create_bookmark(body.into(), session.user_id)
        .await
    {
        Ok(data) => {
            if let (Some(thumbnail_url), None) = (&data.thumbnail_url, &data.archived_url) {
                let mut storage = state.archive_thumbnail_storage.lock().await;

                let url = thumbnail_url.parse().unwrap();
                storage
                    .push(archive_thumbnail::Job {
                        url,
                        bookmark_id: data.id,
                        user_id: session.user_id,
                    })
                    .await
                    .unwrap();
            }

            Ok(CreateResponse::Created(Box::new(data.into())))
        }
        Err(e) => match e {
            bookmark::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

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
