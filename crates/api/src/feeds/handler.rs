use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use colette_core::feeds::FeedsService;

use super::model::CreateFeed;
use crate::{api::Paginated, error::Error, feeds::Feed, session::Session};

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
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let feeds = service
        .list((&session).into())
        .await
        .map(Paginated::<Feed>::from)?;

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
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let feed = service.get(id, (&session).into()).await.map(Feed::from)?;

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
    session: Session,
    Json(body): Json<CreateFeed>,
) -> Result<impl IntoResponse, Error> {
    let feed = service
        .create(body.into(), (&session).into())
        .await
        .map(Feed::from)?;

    Ok(Json(feed))
}
