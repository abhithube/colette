use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;

use super::{SUBSCRIPTIONS_TAG, SubscriptionDetails};
use crate::{
    ApiState,
    common::{ApiError, AuthUser, Id, Path, Query},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id, SubscriptionGetQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "getSubscription",
    description = "Get a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<SubscriptionGetQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .get_subscription(
            subscription::SubscriptionGetQuery {
                id,
                with_feed: query.with_feed,
                with_unread_count: query.with_unread_count,
                with_tags: query.with_tags,
            },
            user_id,
        )
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

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct SubscriptionGetQuery {
    #[serde(default = "with_feed")]
    with_feed: bool,
    #[serde(default = "with_unread_count")]
    with_unread_count: bool,
    #[serde(default = "with_tags")]
    with_tags: bool,
}

fn with_feed() -> bool {
    false
}

fn with_unread_count() -> bool {
    false
}

fn with_tags() -> bool {
    false
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Subscription by ID")]
pub(super) struct OkResponse(SubscriptionDetails);

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

    #[response(status = StatusCode::NOT_FOUND, description = "Subscription not found")]
    NotFound(ApiError),

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
