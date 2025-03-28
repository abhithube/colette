use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::subscription;
use uuid::Uuid;

use super::{SUBSCRIPTIONS_TAG, Subscription};
use crate::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(SubscriptionListQuery),
    responses(ListResponse),
    operation_id = "listSubscriptions",
    description = "List user subscriptions",
    tag = SUBSCRIPTIONS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<SubscriptionListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .subscription_service
        .list_subscriptions(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct SubscriptionListQuery {
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<Uuid>>,
}

impl From<SubscriptionListQuery> for subscription::SubscriptionListQuery {
    fn from(value: SubscriptionListQuery) -> Self {
        Self {
            tags: if value.filter_by_tags.unwrap_or(value.tags.is_some()) {
                value.tags
            } else {
                None
            },
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of subscriptions")]
    Ok(Paginated<Subscription>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
