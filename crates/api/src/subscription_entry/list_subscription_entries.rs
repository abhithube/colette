use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::subscription_entry;
use uuid::Uuid;

use super::{SUBSCRIPTION_ENTRIES_TAG, SubscriptionEntryDetails};
use crate::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(SubscriptionEntryListQuery),
    responses(ListResponse),
    operation_id = "listSubscriptionEntries",
    description = "List subscription entries",
    tag = SUBSCRIPTION_ENTRIES_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<SubscriptionEntryListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .subscription_entry_service
        .list_subscription_entries(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct SubscriptionEntryListQuery {
    #[param(nullable = false)]
    pub stream_id: Option<Uuid>,
    #[param(nullable = false)]
    pub subscription_id: Option<Uuid>,
    #[param(nullable = false)]
    pub has_read: Option<bool>,
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<Uuid>>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<SubscriptionEntryListQuery> for subscription_entry::SubscriptionEntryListQuery {
    fn from(value: SubscriptionEntryListQuery) -> Self {
        Self {
            stream_id: value.stream_id,
            subscription_id: value.subscription_id,
            has_read: value.has_read,
            tags: value.tags,
            cursor: value.cursor,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of subscription entries")]
    Ok(Paginated<SubscriptionEntryDetails>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
