use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::pat::PatError;
use colette_handler::{Handler as _, UpdatePatCommand, UpdatePatError};

use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Json, Path},
    pat::PERSONAL_ACCESS_TOKENS_TAG,
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = PatUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updatePat",
    description = "Update a PAT by ID",
    tag = PERSONAL_ACCESS_TOKENS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Auth { user_id }: Auth,
    Json(body): Json<PatUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .update_pat
        .handle(UpdatePatCommand {
            id: id.into(),
            title: body.title,
            user_id,
        })
        .await
    {
        Ok(_) => Ok(OkResponse),
        Err(e) => match e {
            UpdatePatError::Pat(e) => match e {
                PatError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
                PatError::InvalidTitleLength => Err(ErrResponse::UnprocessableEntity(e.into())),
                _ => Err(ErrResponse::InternalServerError(e.into())),
            },
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Details regarding the existing PAT to update
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct PatUpdate {
    /// Human-readable name for the PAT to update, cannot be empty
    #[schema(min_length = 1, nullable = false)]
    title: Option<String>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully updated PAT")]
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

    #[response(status = StatusCode::NOT_FOUND, description = "PAT not found")]
    NotFound(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
