use crate::{api::Paginated, error::Error, session::SessionAuth};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use colette_core::feeds::FeedsService;
use std::sync::Arc;

use super::model::CreateFeed;

#[axum::debug_handler]
#[utoipa::path(
  get,
  path = "",
  responses(
    (status = 200, description = "Paginated list of feeds", body = FeedList)
  ),
  operation_id = "listFeeds",
  tag = "Feeds"
)]
pub async fn list_feeds(
    State(service): State<Arc<FeedsService>>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let feeds = service.list(session).await?;
    let feeds = Paginated::from(feeds);

    Ok(Json(feeds))
}

#[axum::debug_handler]
#[utoipa::path(
  get,
  path = "/{id}",
  params(
    ("id", description = "Feed ID")
  ),
  responses(
    (status = 200, description = "Feed by ID", body = Feed)
  ),
  operation_id = "getFeed",
  tag = "Feeds"
)]
pub async fn get_feed(
    State(service): State<Arc<FeedsService>>,
    Path(id): Path<String>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let feed = service.get(id, session).await?;

    Ok(Json(feed))
}

#[axum::debug_handler]
#[utoipa::path(
  post,
  path = "",
  request_body = CreateFeed,
  responses(
    (status = 201, description = "Created feed", body = Feed)
  ),
  operation_id = "createFeed",
  tag = "Feeds"
)]
pub async fn create_feed(
    State(service): State<Arc<FeedsService>>,
    SessionAuth(session): SessionAuth,
    Json(body): Json<CreateFeed>,
) -> Result<impl IntoResponse, Error> {
    let feed = service.create(body.into(), session).await?;

    Ok(Json(feed))
}
