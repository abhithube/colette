use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};
use colette_core::library;
use uuid::Uuid;

use super::{FeedTreeItem, LIBRARY_TAG};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "/feedTree",
    params(LibraryItemListQuery),
    responses(ListResponse),
    operation_id = "listFeedTree",
    description = "List user feed tree, consisting of folders and feeds",
    tag = LIBRARY_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<LibraryItemListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<impl IntoResponse, Error> {
    match state
        .library_service
        .list_feed_tree(query.into(), user_id)
        .await
    {
        Ok(data) => Ok(ListResponse::Ok(Paginated {
            data: data.data.into_iter().map(Into::into).collect(),
            cursor: data.cursor,
        })),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct LibraryItemListQuery {
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub cursor: Option<String>,
}

impl From<LibraryItemListQuery> for library::LibraryItemListQuery {
    fn from(value: LibraryItemListQuery) -> Self {
        Self {
            folder_id: value.folder_id,
            cursor: value.cursor,
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of folders and feeds")]
    Ok(Paginated<FeedTreeItem>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
