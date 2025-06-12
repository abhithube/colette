use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::subscription_entry;
use uuid::Uuid;

use super::{SUBSCRIPTION_ENTRIES_TAG, SubscriptionEntryDetails};
use crate::{
    ApiState,
    common::{ApiError, Auth, Paginated, Query},
};

#[utoipa::path(
    get,
    path = "",
    params(SubscriptionEntryListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listSubscriptionEntries",
    description = "List subscription entries",
    tag = SUBSCRIPTION_ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<SubscriptionEntryListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    match state
        .subscription_entry_service
        .list_subscription_entries(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(OkResponse(data.into())),
        Err(e) => Err(ErrResponse::InternalServerError(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub(super) struct SubscriptionEntryListQuery {
    /// Filter by the ID of a stream whose filters may apply to the subscription entry
    #[param(nullable = false)]
    stream_id: Option<Uuid>,
    /// Filter by the ID of the associated subscription
    #[param(nullable = false)]
    subscription_id: Option<Uuid>,
    /// Filter by whether the subscription entry has been marked as read
    #[param(nullable = false)]
    has_read: Option<bool>,
    /// Filter by the IDs of the tags linked to the associated subscription
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    tags: Option<Vec<Uuid>>,
    /// Pagination cursor from the previous set of results
    #[param(nullable = false)]
    cursor: Option<String>,
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

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of subscription entries")]
pub(super) struct OkResponse(Paginated<SubscriptionEntryDetails>);

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
