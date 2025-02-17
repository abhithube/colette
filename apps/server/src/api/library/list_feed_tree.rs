use axum::{
    Json,
    extract::{Query, State},
    response::{IntoResponse, Response},
};

use super::{FeedTreeItem, LIBRARY_TAG};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
    library::TreeListQuery,
};

#[utoipa::path(
    get,
    path = "/feedTree",
    params(TreeListQuery),
    responses(ListResponse),
    operation_id = "listFeedTree",
    description = "List user feed tree, consisting of folders and feeds",
    tag = LIBRARY_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<TreeListQuery>,
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
