use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::tag;

use super::{TAGS_TAG, Tag};
use crate::{
    ApiState,
    common::{ApiError, Auth, Json, NonEmptyString},
};

#[utoipa::path(
  post,
  path = "",
  request_body = TagCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createTag",
  description = "Create a tag",
  tag = TAGS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<TagCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state.tag_service.create_tag(body.into(), user_id).await {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            tag::Error::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Data to create a new tag
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct TagCreate {
    /// Human-readable name for the new tag, cannot be empty
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
}

impl From<TagCreate> for tag::TagCreate {
    fn from(value: TagCreate) -> Self {
        Self {
            title: value.title.into(),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created tag")]
pub(super) struct OkResponse(Tag);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::CONFLICT, description = "Tag already exists")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
