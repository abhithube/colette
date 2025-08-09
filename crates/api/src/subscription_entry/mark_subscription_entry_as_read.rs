use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    subscription_entry::{MarkSubscriptionEntryAsReadCommand, MarkSubscriptionEntryAsReadError},
};
use uuid::Uuid;

use super::SUBSCRIPTION_ENTRIES_TAG;
use crate::{
    ApiState,
    common::{ApiError, Auth, Id, Path},
};

#[utoipa::path(
  post,
  path = "/{id}/markAsRead",
  params(Id),
  responses(OkResponse, ErrResponse),
  operation_id = "markSubscriptionEntryAsRead",
  description = "Mark a subscription entry as read",
  tag = SUBSCRIPTION_ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(id): Path<Uuid>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .mark_subscription_entry_as_read
        .handle(MarkSubscriptionEntryAsReadCommand { id, user_id })
        .await
    {
        Ok(()) => Ok(OkResponse),
        Err(e) => match e {
            MarkSubscriptionEntryAsReadError::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            MarkSubscriptionEntryAsReadError::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::NO_CONTENT, description = "Successfully marked subscription entry as read")]
pub(super) struct OkResponse;

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Subscription entry not found")]
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
