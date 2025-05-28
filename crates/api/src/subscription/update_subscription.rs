use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;

use super::{SUBSCRIPTIONS_TAG, Subscription};
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Id, Json, NonEmptyString, Path},
};

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = SubscriptionUpdate,
    responses(OkResponse, ErrResponse),
    operation_id = "updateSubscription",
    description = "Update a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    AuthUser(user): AuthUser,
    Json(body): Json<SubscriptionUpdate>,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .update_subscription(id, body.into(), user.id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => match e {
            subscription::Error::Forbidden(_) => Err(ErrResponse::Forbidden(e.into())),
            subscription::Error::NotFound(_) => Err(ErrResponse::NotFound(e.into())),
            _ => Err(ErrResponse::InternalServerError(e.into())),
        },
    }
}

/// Updates to make to an existing subscription
#[derive(Debug, Clone, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub(super) struct SubscriptionUpdate {
    /// Human-readable name for the subscription to update, cannot be empty
    #[schema(value_type = Option<String>, min_length = 1)]
    title: Option<NonEmptyString>,
    /// Description for the subscription to update, cannot be empty
    #[serde(default, with = "serde_with::rust::double_option")]
    #[schema(value_type = Option<Option<String>>, min_length = 1)]
    description: Option<Option<NonEmptyString>>,
}

impl From<SubscriptionUpdate> for subscription::SubscriptionUpdate {
    fn from(value: SubscriptionUpdate) -> Self {
        Self {
            title: value.title.map(Into::into),
            description: value.description.map(|e| e.map(Into::into)),
        }
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Updated subscription")]
pub(super) struct OkResponse(Subscription);

impl IntoResponse for OkResponse {
    fn into_response(self) -> Response {
        (StatusCode::OK, axum::Json(self.0)).into_response()
    }
}

#[allow(dead_code)]
#[derive(utoipa::IntoResponses)]
pub(super) enum ErrResponse {
    #[response(status = StatusCode::UNAUTHORIZED, description = "User not authenticated")]
    Unauthorized(ApiError),

    #[response(status = StatusCode::FORBIDDEN, description = "User not authorized")]
    Forbidden(ApiError),

    #[response(status = StatusCode::NOT_FOUND, description = "Subscription not found")]
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
