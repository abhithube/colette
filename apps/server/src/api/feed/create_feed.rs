use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed::{self, FeedService};
use url::Url;
use uuid::Uuid;

use super::{FEEDS_TAG, Feed};
use crate::api::common::{AuthUser, BaseError, Error, NonEmptyString};

#[utoipa::path(
    post,
    path = "",
    request_body = FeedCreate,
    responses(CreateResponse),
    operation_id = "createFeed",
    description = "Subscribe to a web feed",
    tag = FEEDS_TAG
  )]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<FeedService>>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<FeedCreate>,
) -> Result<CreateResponse, Error> {
    match service.create_feed(body.into(), user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            feed::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedCreate {
    pub url: Url,
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub folder_id: Option<Uuid>,
    #[schema(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<FeedCreate> for feed::FeedCreate {
    fn from(value: FeedCreate) -> Self {
        Self {
            url: value.url,
            title: value.title.into(),
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created feed")]
    Created(Feed),

    #[response(status = 409, description = "Feed not cached")]
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
