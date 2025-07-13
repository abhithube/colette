use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription::{self, SubscriptionCursor};
use uuid::Uuid;

use super::{SUBSCRIPTIONS_TAG, SubscriptionDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(SubscriptionListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listSubscriptions",
    description = "List user subscriptions",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<SubscriptionListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_service
        .list_subscriptions(query.try_into()?, user_id)
        .await
    {
        Ok(subscriptions) => {
            let data = subscriptions
                .try_into()
                .map_err(ErrResponse::InternalServerError)?;

            Ok(OkResponse(data))
        }
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct SubscriptionListQuery {
    /// Whether to filter by tags linked to the subscription
    #[param(nullable = false)]
    filter_by_tags: Option<bool>,
    /// Filter by the IDs of the tags linked to the subscription
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    tags: Option<Vec<Uuid>>,
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
    /// Whether to include the feed associated with the subscription
    #[serde(default = "with_feed")]
    with_feed: bool,
    /// Whether to include the count of the unread subscription entries associated with the subscription
    #[serde(default = "with_unread_count")]
    with_unread_count: bool,
    /// Whether to include the tags linked to the subscription
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

impl TryFrom<SubscriptionListQuery> for subscription::SubscriptionListQuery {
    type Error = ErrResponse;

    fn try_from(value: SubscriptionListQuery) -> Result<Self, Self::Error> {
        let cursor = value
            .cursor
            .map(|e| decode_cursor::<SubscriptionCursor>(&e))
            .transpose()
            .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

        Ok(Self {
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
            cursor,
            limit: Some(PAGINATION_LIMIT),
            with_feed: value.with_feed,
            with_unread_count: value.with_unread_count,
            with_tags: value.with_tags,
        })
    }
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of subscriptions")]
pub(super) struct OkResponse(Paginated<SubscriptionDetails>);

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

    #[response(status = "default", description = "Unknown error")]
    InternalServerError(ApiError),
}

impl IntoResponse for ErrResponse {
    fn into_response(self) -> Response {
        match self {
            Self::InternalServerError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ApiError::unknown()).into_response()
            }
            _ => unreachable!(),
        }
    }
}
