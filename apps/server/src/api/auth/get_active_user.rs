use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use colette_core::auth::AuthService;

use super::{AUTH_TAG, User};
use crate::api::common::{AuthUser, Error};

#[utoipa::path(
  get,
  path = "/@me",
  responses(GetActiveResponse),
  operation_id = "getActiveUser",
  description = "Get the active user",
  tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(service): State<Arc<AuthService>>,
    AuthUser(user_id): AuthUser,
) -> Result<GetActiveResponse, Error> {
    match service.get_active(user_id).await {
        Ok(data) => Ok(GetActiveResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetActiveResponse {
    #[response(status = 200, description = "Active user")]
    Ok(User),
}

impl IntoResponse for GetActiveResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
