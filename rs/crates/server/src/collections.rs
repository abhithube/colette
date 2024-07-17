use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::collections::{self, CollectionsService};
use uuid::Uuid;

use crate::common::{BaseError, CollectionList, Context, Error, Id, Paginated, Session};

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_collections, get_collection, create_collection, delete_collection),
    components(schemas(Collection, CreateCollection))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/collections",
            Router::new()
                .route("/", routing::get(list_collections).post(create_collection))
                .route(
                    "/:id",
                    routing::get(get_collection).delete(delete_collection),
                ),
        )
    }
}

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
            collections::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
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
            collections::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub profile_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<colette_core::Collection> for Collection {
    fn from(value: colette_core::Collection) -> Self {
        Self {
            id: value.id,
            title: value.title,
            profile_id: value.profile_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollection {
    #[schema(min_length = 1)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: String,
}

impl From<CreateCollection> for collections::CreateCollection {
    fn from(value: CreateCollection) -> Self {
        Self { title: value.title }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of collections")]
    Ok(CollectionList),
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created collection")]
    Created(Collection),

    #[allow(dead_code)]
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

#[derive(Debug, utoipa::IntoResponses)]
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
