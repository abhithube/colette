use axum::{
    Json,
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use colette_core::stream;

use super::STREAMS_TAG;
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Id, Paginated},
    feed_entry::FeedEntry,
};

#[utoipa::path(
    get,
    path = "/{id}/entries",
    params(Id, StreamEntryListQuery),
    responses(ListResponse),
    operation_id = "listStreamEntries",
    description = "List stream entries",
    tag = STREAMS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Path(Id(id)): Path<Id>,
    Query(query): Query<StreamEntryListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state
        .stream_service
        .list_stream_entries(id, query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct StreamEntryListQuery {
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<StreamEntryListQuery> for stream::StreamEntryListQuery {
    fn from(value: StreamEntryListQuery) -> Self {
        Self {
            cursor: value.cursor,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of stream entries")]
    Ok(Paginated<FeedEntry>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
