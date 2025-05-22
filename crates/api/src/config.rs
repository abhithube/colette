use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};

use super::ApiState;
use crate::ApiConfig;

const CONFIG_TAG: &str = "Config";

#[derive(OpenApi)]
#[openapi(components(schemas(ApiConfig)))]
pub(crate) struct ConfigApi;

impl ConfigApi {
    pub(crate) fn router() -> OpenApiRouter<ApiState> {
        OpenApiRouter::with_openapi(ConfigApi::openapi()).routes(routes!(handler))
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(OkResponse),
    operation_id = "getConfig",
    description = "Get the API config",
    security(()),
    tag = CONFIG_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(State(state): State<ApiState>) -> OkResponse {
    OkResponse(state.config)
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "API config")]
pub(super) struct OkResponse(ApiConfig);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, Json(self.0)).into_response()
    }
}
