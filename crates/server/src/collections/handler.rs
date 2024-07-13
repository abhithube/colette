use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_valid::Valid;
use colette_core::collections::{self, CollectionsService};

use super::{
    model::{CreateCollection, CreateResponse, DeleteResponse, GetResponse, ListResponse},
    Collection,
};
use crate::{
    common::{self, Id, Paginated},
    error::Error,
    session::Session,
};

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listCollections",
    description = "List the active profile collections",
    tag = "Collections"
)]
#[axum::debug_handler]
pub async fn list_collections(
    State(service): State<Arc<CollectionsService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(session.into())
        .await
        .map(Paginated::<Collection>::from);

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
    operation_id = "getCollection",
    description = "Get a collection by ID",
    tag = "Collections"
)]
#[axum::debug_handler]
pub async fn get_collection(
    State(service): State<Arc<CollectionsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.get(id, session.into()).await.map(Collection::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            collections::Error::NotFound(_) => Ok(GetResponse::NotFound(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CreateCollection,
  responses(CreateResponse),
  operation_id = "createCollection",
  description = "Create a bookmarks collection",
  tag = "Collections"
)]
#[axum::debug_handler]
pub async fn create_collection(
    State(service): State<Arc<CollectionsService>>,
    session: Session,
    Valid(Json(body)): Valid<Json<CreateCollection>>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .create(body.into(), session.into())
        .await
        .map(Collection::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(_) => Err(Error::Unknown),
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteCollection",
    description = "Delete a collection by ID",
    tag = "Collections"
)]
#[axum::debug_handler]
pub async fn delete_collection(
    State(service): State<Arc<CollectionsService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.delete(id, session.into()).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            collections::Error::NotFound(_) => Ok(DeleteResponse::NotFound(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}
