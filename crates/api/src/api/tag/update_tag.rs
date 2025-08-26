use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_crud::TagError;
use colette_handler::{Handler as _, UpdateTagCommand, UpdateTagError};

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Id, Json, NonEmptyString, Path},
    tag::TAGS_TAG,
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = TagUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateTag",
    description = "Update a tag by ID",
    tag = TAGS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<TagUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .update_tag
        .handle(UpdateTagCommand {
            id: id.into(),
            title: body.title.map(Into::into),
            user_id,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            UpdateTagError::Tag(TagError::NotFound(_)) => Err(ErrResponse::NotFound(e.into())),
            UpdateTagError::Tag(TagError::Conflict(_)) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Updates to make to an existing tag
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct TagUpdate {
    /// Human-readable name for the tag to update, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully updated tag")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Tag not found")]
    NotFound(ApiError),

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
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
