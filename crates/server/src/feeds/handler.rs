use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use colette_core::feeds::{self, FeedsService};

use super::model::{CreateFeed, CreateResponse, DeleteResponse, GetResponse, ListResponse};
use crate::{
    common::{self, Id, Paginated},
    error::Error,
    feeds::Feed,
    session::Session,
};

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listFeeds",
    description = "List the active profile feeds",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn list_feeds(
    State(service): State<Arc<FeedsService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(session.into())
        .await
        .map(Paginated::<Feed>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFeed",
    description = "Get a feed by ID",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn get_feed(
    State(service): State<Arc<FeedsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.get(id, session.into()).await.map(Feed::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(GetResponse::NotFound(common::Error {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CreateFeed,
  responses(CreateResponse),
  operation_id = "createFeed",
  description = "Subscribe to a web feed",
  tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn create_feed(
    State(service): State<Arc<FeedsService>>,
    session: Session,
    Json(body): Json<CreateFeed>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .create(body.into(), session.into())
        .await
        .map(Feed::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(e) => match e {
            feeds::Error::Scraper(_) => Ok(CreateResponse::BadGateway(common::Error {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteFeed",
    description = "Delete a feed by ID",
    tag = "Feeds"
)]
#[axum::debug_handler]
pub async fn delete_feed(
    State(service): State<Arc<FeedsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.delete(id, session.into()).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            feeds::Error::NotFound(_) => Ok(DeleteResponse::NotFound(common::Error {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}
