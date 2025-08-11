use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    tag::{TagError, UpdateTagCommand, UpdateTagError},
};

use crate::{
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
            UpdateTagError::Core(TagError::Forbidden(_)) => Err(ErrResponse::Forbidden(e.into())),
            UpdateTagError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
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

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Tag not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
