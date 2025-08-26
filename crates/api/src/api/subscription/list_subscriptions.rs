use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_handler::{Handler as _, ListSubscriptionsQuery, SubscriptionCursor};
use uuid::Uuid;

use crate::api::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
    subscription::{SUBSCRIPTIONS_TAG, Subscription},
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
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<SubscriptionCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_subscriptions
        .handle(ListSubscriptionsQuery {
            tags: if query.filter_by_tags.unwrap_or(query.tags.is_some()) {
                query.tags
            } else {
                None
            },
            cursor,
            limit: Some(PAGINATION_LIMIT),
            user_id: user_id.as_inner(),
        })
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
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of subscriptions")]
pub(super) struct OkResponse(Paginated<Subscription>);

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
