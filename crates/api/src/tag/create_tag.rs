use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag;

use super::{TAGS_TAG, Tag};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = TagCreate,
  responses(CreateResponse),
  operation_id = "createTag",
  description = "Create a tag",
  tag = TAGS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<TagCreate>,
) -> Result<CreateResponse, Error> {
    match state.tag_service.create_tag(body.into(), user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            tag::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TagCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
}

impl From<TagCreate> for tag::TagCreate {
    fn from(value: TagCreate) -> Self {
        Self {
            title: value.title.into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created tag")]
    Created(Tag),

    #[response(status = 409, description = "Tag already exists")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
