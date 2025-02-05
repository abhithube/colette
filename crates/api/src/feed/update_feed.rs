use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{
    common::NonEmptyString,
    feed::{self, FeedService},
};
use uuid::Uuid;

use super::Feed;
use crate::common::{BaseError, Error, Id, Session, FEEDS_TAG};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FeedUpdate {
    #[schema(value_type = Option<String>, min_length = 1)]
    pub title: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub folder_id: Option<Option<Uuid>>,
    #[schema(value_type = Option<Vec<String>>, nullable = false, min_length = 1)]
    pub tags: Option<Vec<NonEmptyString>>,
}

impl From<FeedUpdate> for feed::FeedUpdate {
    fn from(value: FeedUpdate) -> Self {
        Self {
            title: value.title,
            folder_id: value.folder_id,
            tags: value.tags,
        }
    }
}

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
    State(service): State<Arc<FeedService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FeedUpdate>,
) -> Result<UpdateResponse, Error> {
    match service.update_feed(id, body.into(), session.user_id).await {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            feed::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

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
