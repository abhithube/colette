use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{subscription, subscription_entry};
use uuid::Uuid;

use super::SUBSCRIPTIONS_TAG;
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Path},
    subscription_entry::SubscriptionEntry,
};

#[utoipa::path(
  post,
  path = "/{sid}/entries/{eid}/markAsUnread",
  params(
    ("sid" = Uuid, Path),
    ("eid" = Uuid, Path),
  ),
  responses(OkResponse, ErrResponse),
  operation_id = "markSubscriptionEntryAsUnread",
  description = "Mark a subscription entry as unread",
  tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path((subscription_id, feed_entry_id)): Path<(Uuid, Uuid)>,
    AuthUser(user): AuthUser,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .mark_subscription_entry_as_unread(subscription_id, feed_entry_id, user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            subscription::Error::SubscriptionEntry(subscription_entry::Error::Forbidden(_)) => {
                Err(ErrResponse::Forbidden(e.into()))
            }
            subscription::Error::SubscriptionEntry(subscription_entry::Error::NotFound(_)) => {
                Err(ErrResponse::NotFound(e.into()))
            }
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Updated subscription entry")]
pub(super) struct OkResponse(SubscriptionEntry);

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
