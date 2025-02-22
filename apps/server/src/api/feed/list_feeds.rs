use axum::{
    Json,
    extract::State,
    response::{IntoResponse, Response},
};
use axum_extra::extract::Query;
use colette_core::feed;
use uuid::Uuid;

use super::{FEEDS_TAG, Feed};
use crate::api::{
    ApiState,
    common::{AuthUser, Error, Paginated},
};

#[utoipa::path(
    get,
    path = "",
    params(FeedListQuery),
    responses(ListResponse),
    operation_id = "listFeeds",
    description = "List user feeds",
    tag = FEEDS_TAG
)]
#[axum::debug_handler]
pub async fn handler(
    State(state): State<ApiState>,
    Query(query): Query<FeedListQuery>,
    AuthUser(user_id): AuthUser,
) -> Result<ListResponse, Error> {
    match state.feed_service.list_feeds(query.into(), user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[derive(Debug, Clone, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FeedListQuery {
    #[param(nullable = false)]
    pub filter_by_folder: Option<bool>,
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<String>>,
}

impl From<FeedListQuery> for feed::FeedListQuery {
    fn from(value: FeedListQuery) -> Self {
        Self {
            folder_id: if value.filter_by_folder.unwrap_or(value.folder_id.is_some()) {
                Some(value.folder_id)
            } else {
                None
            },
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
    #[response(status = 200, description = "Paginated list of feeds")]
    Ok(Paginated<Feed>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}
