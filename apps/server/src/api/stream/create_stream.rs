use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::{STREAMS_TAG, Stream};
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error, NonEmptyString},
    feed_entry::FeedEntryFilter,
};

#[utoipa::path(
  post,
  path = "",
  request_body = StreamCreate,
  responses(CreateResponse),
  operation_id = "createStream",
  description = "Create a stream",
  tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    Json(body): Json<StreamCreate>,
) -> Result<CreateResponse, Error> {
    match state
        .stream_service
        .create_stream(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            stream::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StreamCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub filter: FeedEntryFilter,
}

impl From<StreamCreate> for stream::StreamCreate {
    fn from(value: StreamCreate) -> Self {
        Self {
            title: value.title.into(),
            filter: value.filter.into(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created stream")]
    Created(Stream),

    #[response(status = 409, description = "Stream already exists")]
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
