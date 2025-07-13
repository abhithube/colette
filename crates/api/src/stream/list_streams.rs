use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::stream::{self, StreamCursor};

use super::{STREAMS_TAG, Stream};
use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(StreamListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listStreams",
    description = "List user streams",
    tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<StreamListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .stream_service
        .list_streams(query.try_into()?, user_id)
        .await
    {
        Ok(streams) => {
            let data = streams
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct StreamListQuery {
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

impl TryFrom<StreamListQuery> for stream::StreamListQuery {
    type Error = ErrResponse;

    fn try_from(value: StreamListQuery) -> Result<Self, Self::Error> {
        let cursor = value
            .cursor
            .map(|e| decode_cursor::<StreamCursor>(&e))
            .transpose()
            .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

        Ok(Self {
            cursor,
            limit: Some(PAGINATION_LIMIT),
        })
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of streams")]
pub(super) struct OkResponse(Paginated<Stream>);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

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
