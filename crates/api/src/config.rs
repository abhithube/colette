use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing,
};
use utoipa::OpenApi;

use crate::{ApiConfig, ApiState};

const CONFIG_TAG: &str = "Config";

#[derive(OpenApi)]
#[openapi(components(schemas(ApiConfig)), paths(handler))]
pub(crate) struct ConfigApi;

impl ConfigApi {
    pub(crate) fn router() -> Router<ApiState> {
        Router::new().route("/", routing::get(handler))
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
