use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{subscription, subscription_entry};
use uuid::Uuid;

use super::SUBSCRIPTIONS_TAG;
use crate::api::{
    ApiState,
    common::{AuthUser, BaseError, Error},
    subscription_entry::SubscriptionEntry,
};

#[utoipa::path(
  post,
  path = "/{sid}/entries/{eid}/markAsUnread",
  params(
    ("sid" = Uuid, Path),
    ("eid" = Uuid, Path),
  ),
  responses(UpdateResponse),
  operation_id = "markSubscriptionEntryAsUnread",
  description = "Mark a subscription entry as unread",
  tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path((subscription_id, feed_entry_id)): Path<(Uuid, Uuid)>,
    AuthUser(user_id): AuthUser,
) -> Result<UpdateResponse, Error> {
    match state
        .subscription_service
        .mark_subscription_entry_as_unread(feed_entry_id, subscription_id, user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            subscription::Error::SubscriptionEntry(subscription_entry::Error::Forbidden(_)) => {
                Ok(UpdateResponse::Forbidden(BaseError {
                    message: e.to_string(),
                }))
            }
            subscription::Error::SubscriptionEntry(subscription_entry::Error::NotFound(_)) => {
                Ok(UpdateResponse::NotFound(BaseError {
                    message: e.to_string(),
                }))
            }
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[allow(dead_code, clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated subscription entry")]
    Ok(SubscriptionEntry),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Subscription entry not found")]
    NotFound(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
