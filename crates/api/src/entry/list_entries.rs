use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    Handler as _,
    entry::{EntryCursor, ListEntriesQuery},
};
use uuid::Uuid;

use crate::{
    ApiState,
    common::{ApiError, Auth, Query},
    entry::{ENTRIES_TAG, Entry},
    pagination::{PAGINATION_LIMIT, Paginated, decode_cursor},
};

#[utoipa::path(
    get,
    path = "",
    params(EntryListQuery),
    responses(OkResponse, ErrResponse),
    operation_id = "listEntries",
    description = "List user entries",
    tag = ENTRIES_TAG
)]
#[axum::debug_handler]
pub(super) async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<EntryListQuery>,
    Auth { user_id }: Auth,
) -> Result<OkResponse, ErrResponse> {
    let cursor = query
        .cursor
        .map(|e| decode_cursor::<EntryCursor>(&e))
        .transpose()
        .map_err(|e| ErrResponse::InternalServerError(e.into()))?;

    match state
        .list_entries
        .handle(ListEntriesQuery {
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
        Ok(entries) => {
            let data = entries
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
pub(super) struct EntryListQuery {
    /// Filter by the ID of the associated subscription
    #[param(nullable = false)]
    subscription_id: Option<Uuid>,
    /// Filter by whether the entry has been marked as read
    #[param(nullable = false)]
    has_read: Option<bool>,
    /// Filter by the IDs of the tags linked to the associated subscription
    #[param(nullable = false)]
    #[serde(rename = "tag[]")]
    tags: Option<Vec<Uuid>>,
    /// Filter by the ID of a collection whose filters may apply to the entry
    #[param(nullable = false)]
    collection_id: Option<Uuid>,
    /// Pagination cursor
    #[param(nullable = false)]
    cursor: Option<String>,
}

#[derive(utoipa::IntoResponses)]
#[response(status = StatusCode::OK, description = "Paginated list of entries")]
pub(super) struct OkResponse(Paginated<Entry>);

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
