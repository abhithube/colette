use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    common::NonEmptyString,
    tag::{self, TagService},
};

use super::Tag;
use crate::common::{BaseError, Error, Id, Session, TAGS_TAG};

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
}

impl From<TagUpdate> for tag::TagUpdate {
    fn from(value: TagUpdate) -> Self {
        Self { title: value.title }
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = TagUpdate,
    responses(UpdateResponse),
    operation_id = "updateTag",
    description = "Update a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<TagService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<TagUpdate>,
) -> Result<UpdateResponse, Error> {
    match service.update_tag(id, body.into(), session.user_id).await {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            tag::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated tag")]
    Ok(Tag),

    #[response(status = 404, description = "Tag not found")]
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
