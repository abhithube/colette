use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use colette_handler::{CreatePatCommand, Handler as _};
use uuid::Uuid;

use crate::{
    ApiState,
    common::{ApiError, Auth, Json},
    pat::PERSONAL_ACCESS_TOKENS_TAG,
};

#[utoipa::path(
  post,
  path = "",
  request_body = PatCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createPat",
  description = "Create a PAT",
  tag = PERSONAL_ACCESS_TOKENS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<PatCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .create_pat
        .handle(CreatePatCommand {
            title: body.title,
            user_id,
        })
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

/// Data to create a new API key
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct PatCreate {
    /// Human-readable name for the API key to create, cannot be empty
    #[schema(min_length = 1, max_length = 50)]
    title: String,
}

/// Newly created API key, containing the full value. This value must be saved in a safe location, as subsequent GET requests will only show a preview.
#[derive(Debug, Clone, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct PatCreated {
    /// Unique identifier of the new API key
    id: Uuid,
    /// Full value of the new API key
    value: String,
    /// Human-readable name of the new API key
    title: String,
    /// Timestamp at which the API key was created
    created_at: DateTime<Utc>,
}

impl From<colette_handler::PatCreated> for PatCreated {
    fn from(value: colette_handler::PatCreated) -> Self {
        Self {
            id: value.id().as_inner(),
            value: value.value().as_inner().to_owned(),
            title: value.title().as_inner().to_owned(),
            created_at: value.created_at(),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created API key")]
pub(super) struct OkResponse(PatCreated);

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

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
