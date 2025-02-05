use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::Query;
use colette_core::{
    common::NonEmptyString,
    feed::{self, FeedService},
};
use uuid::Uuid;

use super::Feed;
use crate::{
    common::{Error, Session, FEEDS_TAG},
    Paginated,
};

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct FeedListQuery {
    #[param(nullable = false)]
    pub filter_by_folder: Option<bool>,
    pub folder_id: Option<Uuid>,
    #[param(nullable = false)]
    pub filter_by_tags: Option<bool>,
    #[param(value_type = Option<Vec<String>>, min_length = 1, nullable = false)]
    #[serde(rename = "tag[]")]
    pub tags: Option<Vec<NonEmptyString>>,
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
    State(service): State<Arc<FeedService>>,
    Query(query): Query<FeedListQuery>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_feeds(query.into(), session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
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
