use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::{STREAMS_TAG, Stream};
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Json, NonEmptyString},
    subscription_entry::SubscriptionEntryFilter,
};

#[utoipa::path(
  post,
  path = "",
  request_body = StreamCreate,
  responses(OkResponse, ErrResponse),
  operation_id = "createStream",
  description = "Create a stream",
  tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    AuthUser(user): AuthUser,
    Json(body): Json<StreamCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .stream_service
        .create_stream(body.into(), user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            stream::Error::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct StreamCreate {
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
    filter: SubscriptionEntryFilter,
}

impl From<StreamCreate> for stream::StreamCreate {
    fn from(value: StreamCreate) -> Self {
        Self {
            title: value.title.into(),
            filter: value.filter.into(),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created stream")]
pub(super) struct OkResponse(Stream);

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

    #[response(status = StatusCode::CONFLICT, description = "Stream already exists")]
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
