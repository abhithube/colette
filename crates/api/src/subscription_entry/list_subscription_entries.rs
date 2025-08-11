use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    subscription_entry::{ListSubscriptionEntriesQuery, SubscriptionEntryCursor},
};
use uuid::Uuid;

use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
    subscription_entry::{SUBSCRIPTION_ENTRIES_TAG, SubscriptionEntryDetails},
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
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<SubscriptionEntryCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_subscription_entries
        .handle(ListSubscriptionEntriesQuery {
            collection_id: query.collection_id.map(Into::into),
            subscription_id: query.subscription_id.map(Into::into),
            has_read: query.has_read,
            tags: query.tags.map(|e| e.into_iter().map(Into::into).collect()),
            cursor,
            limit: Some(PAGINATION_LIMIT),
            user_id,
        })
        .await
    {
        Ok(subscription_entries) => {
            let data = subscription_entries
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
pub(super) struct SubscriptionEntryListQuery {
    /// Filter by the ID of a collection whose filters may apply to the subscription entry
    #[param(nullable = false)]
    collection_id: Option<Uuid>,
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
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
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
