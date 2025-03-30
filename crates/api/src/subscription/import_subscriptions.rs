use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use bytes::Bytes;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{AuthUser, Error},
};

#[utoipa::path(
  post,
  path = "/import",
  request_body = Vec<u8>,
  responses(ImportSubscriptionsResponse),
  operation_id = "importSubscriptions",
  description = "Import subscriptions into user account",
  tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    AuthUser(user_id): AuthUser,
    bytes: Bytes,
) -> Result<ImportSubscriptionsResponse, Error> {
    match state
        .subscription_service
        .import_subscriptions(bytes, user_id)
        .await
    {
        Ok(_) => Ok(ImportSubscriptionsResponse::NoContent),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ImportSubscriptionsResponse {
    #[response(status = 204, description = "Successfully started import")]
    NoContent,
}

impl IntoResponse for ImportSubscriptionsResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
        }
    }
}
