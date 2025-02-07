use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use colette_core::{
    collection::{self, CollectionService},
    common::NonEmptyString,
};
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    Paginated,
    common::{BaseError, COLLECTIONS_TAG, Error, Id, Session},
};

#[derive(Clone, axum::extract::FromRef)]
pub struct CollectionState {
    service: Arc<CollectionService>,
}

impl CollectionState {
    pub fn new(service: Arc<CollectionService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Collection, Paginated<Collection>, CollectionCreate, CollectionUpdate)))]
pub struct CollectionApi;

impl CollectionApi {
    pub fn router() -> OpenApiRouter<CollectionState> {
        OpenApiRouter::with_openapi(CollectionApi::openapi())
            .routes(routes!(list_collections, create_collection))
            .routes(routes!(
                get_collection,
                update_collection,
                delete_collection
            ))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
}

impl From<colette_core::Collection> for Collection {
    fn from(value: colette_core::Collection) -> Self {
        Self {
            id: value.id,
            title: value.title,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
}

impl From<CollectionCreate> for collection::CollectionCreate {
    fn from(value: CollectionCreate) -> Self {
        Self { title: value.title }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CollectionUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
}

impl From<CollectionUpdate> for collection::CollectionUpdate {
    fn from(value: CollectionUpdate) -> Self {
        Self { title: value.title }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listCollections",
    description = "List user collections",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn list_collections(
    State(service): State<Arc<CollectionService>>,
    session: Session,
) -> Result<ListResponse, Error> {
    match service.list_collections(session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getCollection",
    description = "Get a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn get_collection(
    State(service): State<Arc<CollectionService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<GetResponse, Error> {
    match service.get_collection(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CollectionCreate,
  responses(CreateResponse),
  operation_id = "createCollection",
  description = "Create a collection",
  tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn create_collection(
    State(service): State<Arc<CollectionService>>,
    session: Session,
    Json(body): Json<CollectionCreate>,
) -> Result<CreateResponse, Error> {
    match service
        .create_collection(body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = CollectionUpdate,
    responses(UpdateResponse),
    operation_id = "updateCollection",
    description = "Update a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn update_collection(
    State(service): State<Arc<CollectionService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<CollectionUpdate>,
) -> Result<UpdateResponse, Error> {
    match service
        .update_collection(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    delete,
    path = "/{id}",
    params(Id),
    responses(DeleteResponse),
    operation_id = "deleteCollection",
    description = "Delete a collection by ID",
    tag = COLLECTIONS_TAG
)]
#[axum::debug_handler]
pub async fn delete_collection(
    State(service): State<Arc<CollectionService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<DeleteResponse, Error> {
    match service.delete_collection(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of collections")]
    Ok(Paginated<Collection>),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum GetResponse {
    #[response(status = 200, description = "Collection by ID")]
    Ok(Collection),

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),
}

impl IntoResponse for GetResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created collection")]
    Created(Collection),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated collection")]
    Ok(Collection),

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for UpdateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted collection")]
    NoContent,

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
        }
    }
}
