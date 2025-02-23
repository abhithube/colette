use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::feed;

use super::{FEEDS_TAG, Feed};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = FeedUpdate,
    responses(UpdateResponse),
    operation_id = "updateFeed",
    description = "Update a feed by ID",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<FeedUpdate>,
) -> Result<UpdateResponse, Error> {
    match state
        .feed_service
        .update_feed(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedUpdate {
    #[schema(value_type = Option<String>, min_length = 1)]
    pub title: Option<NonEmptyString>,
    #[schema(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<FeedUpdate> for feed::FeedUpdate {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
        }
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated feed")]
    Ok(Feed),

    #[response(status = 404, description = "Feed not found")]
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
