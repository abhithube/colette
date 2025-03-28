use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::{STREAMS_TAG, Stream};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id, NonEmptyString},
    subscription_entry::SubscriptionEntryFilter,
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = StreamUpdate,
    responses(UpdateResponse),
    operation_id = "updateStream",
    description = "Update a stream by ID",
    tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<StreamUpdate>,
) -> Result<UpdateResponse, Error> {
    match state
        .stream_service
        .update_stream(id, body.into(), user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            stream::Error::Forbidden(_) => Ok(UpdateResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            stream::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    pub filter: Option<SubscriptionEntryFilter>,
}

impl From<StreamUpdate> for stream::StreamUpdate {
    fn from(value: StreamUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            filter: value.filter.map(Into::into),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated stream")]
    Ok(Stream),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Stream not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
