use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription;

use super::{SUBSCRIPTIONS_TAG, SubscriptionDetails};
use crate::{
    ApiState,
    common::{AuthUser, BaseError, Error, Id},
};

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id, SubscriptionGetQuery),
    responses(GetResponse),
    operation_id = "getSubscription",
    description = "Get a subscription by ID",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<SubscriptionGetQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<GetResponse, Error> {
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
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            subscription::Error::Forbidden(_) => Ok(GetResponse::Forbidden(BaseError {
                message: e.to_string(),
            })),
            subscription::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct SubscriptionGetQuery {
    #[serde(default = "with_feed")]
    pub with_feed: bool,
    #[serde(default = "with_unread_count")]
    pub with_unread_count: bool,
    #[serde(default = "with_tags")]
    pub with_tags: bool,
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

#[allow(clippy::large_enum_variant)]
#[derive(Debug, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Subscription by ID")]
    Ok(SubscriptionDetails),

    #[response(status = 403, description = "User not authorized")]
    Forbidden(BaseError),

    #[response(status = 404, description = "Subscription not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Forbidden(e) => (StatusCode::FORBIDDEN, e).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
