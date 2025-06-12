use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;
use uuid::Uuid;

use super::{SUBSCRIPTIONS_TAG, Subscription};
use crate::{
    ApiState,
    common::{ApiError, Auth, Json, NonEmptyString},
};

#[utoipa::path(
    post,
    path = "",
    request_body = SubscriptionCreate,
    responses(OkResponse, ErrResponse),
    operation_id = "createSubscription",
    description = "Subscribe to a web feed",
    tag = SUBSCRIPTIONS_TAG
  )]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Auth { user_id }: Auth,
    Json(body): Json<SubscriptionCreate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .create_subscription(body.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            subscription::Error::Conflict(_) => Err(ErrResponse::Conflict(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Data to create a new user subscription
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubscriptionCreate {
    /// Human-readable name for the new subscription, cannot be empty
    #[schema(value_type = String, min_length = 1)]
    title: NonEmptyString,
    /// Description for the new subscription, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1)]
    description: Option<NonEmptyString>,
    /// Unique identifier of the associated RSS feed
    feed_id: Uuid,
}

impl From<SubscriptionCreate> for subscription::SubscriptionCreate {
    fn from(value: SubscriptionCreate) -> Self {
        Self {
            title: value.title.into(),
            description: value.description.map(Into::into),
            feed_id: value.feed_id,
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::CREATED, description = "Created subscription")]
pub(super) struct OkResponse(Subscription);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::CREATED, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::CONFLICT, description = "Subscription already exists")]
    Conflict(ApiError),

    #[response(status = StatusCode::UNPROCESSABLE_ENTITY, description = "Invalid input")]
    UnprocessableEntity(ApiError),

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
