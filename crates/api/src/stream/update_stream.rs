use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::{STREAMS_TAG, Stream};
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Id, Json, NonEmptyString, Path},
    subscription_entry::SubscriptionEntryFilter,
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = StreamUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateStream",
    description = "Update a stream by ID",
    tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user): AuthUser,
    Json(body): Json<StreamUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .stream_service
        .update_stream(id, body.into(), user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            stream::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            stream::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct StreamUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    title: Option<NonEmptyString>,
    filter: Option<SubscriptionEntryFilter>,
}

impl From<StreamUpdate> for stream::StreamUpdate {
    fn from(value: StreamUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            filter: value.filter.map(Into::into),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Updated stream")]
pub(super) struct OkResponse(Stream);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Stream not found")]
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
