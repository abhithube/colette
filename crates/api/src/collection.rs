use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use colette_core::collection::{
    self, CollectionCreateData, CollectionRepository, CollectionUpdateData,
};
use uuid::Uuid;

use crate::common::{BaseError, CollectionList, Error, Id, Paginated, Session};

#[derive(Clone, axum::extract::FromRef)]
pub struct CollectionState {
    pub repository: Arc<dyn CollectionRepository>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_collections,
        get_collection,
        create_collection,
        update_collection,
        delete_collection
    ),
    components(schemas(Collection, CollectionList, CollectionCreate, CollectionUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<CollectionState> {
        Router::new().nest(
            "/collections",
            Router::new()
                .route("/", routing::get(list_collections).post(create_collection))
                .route(
                    "/:id",
                    routing::get(get_collection)
                        .patch(update_collection)
                        .delete(delete_collection),
                ),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    #[schema(required)]
    pub folder_id: Option<Uuid>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    bookmark_count: Option<i64>,
}

impl From<colette_core::Collection> for Collection {
    fn from(value: colette_core::Collection) -> Self {
        Self {
            id: value.id,
            title: value.title,
            folder_id: value.folder_id,
            bookmark_count: value.bookmark_count,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CollectionCreate {
    #[schema(min_length = 1)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: String,
    #[schema(required)]
    pub folder_id: Option<Uuid>,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct CollectionUpdate {
    #[schema(min_length = 1, nullable = false)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub title: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub folder_id: Option<Option<Uuid>>,
}

impl From<CollectionUpdate> for CollectionUpdateData {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title,
            folder_id: value.folder_id,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listCollections",
    description = "List the active profile collections"
)]
#[axum::debug_handler]
pub async fn list_collections(
    State(repository): State<Arc<dyn CollectionRepository>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_many_collections(session.profile_id, None, None)
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
    description = "Get a collection by ID"
)]
#[axum::debug_handler]
pub async fn get_collection(
    State(repository): State<Arc<dyn CollectionRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find_one_collection(id, session.profile_id)
        .await
        .map(Collection::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CollectionCreate,
  responses(CreateResponse),
  operation_id = "createCollection",
  description = "Create a collection"
)]
#[axum::debug_handler]
pub async fn create_collection(
    State(repository): State<Arc<dyn CollectionRepository>>,
    session: Session,
    Valid(Json(body)): Valid<Json<CollectionCreate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .create_collection(CollectionCreateData {
            title: body.title,
            folder_id: body.folder_id,
            profile_id: session.profile_id,
        })
        .await
        .map(Collection::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
        Err(e) => match e {
            collection::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    patch,
    path = "/{id}",
    params(Id),
    request_body = CollectionUpdate,
    responses(UpdateResponse),
    operation_id = "updateCollection",
    description = "Update a collection by ID"
)]
#[axum::debug_handler]
pub async fn update_collection(
    State(repository): State<Arc<dyn CollectionRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Valid(Json(body)): Valid<Json<CollectionUpdate>>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update_collection(id, session.profile_id, body.into())
        .await
        .map(Collection::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    operation_id = "deleteCollection",
    description = "Delete a collection by ID"
)]
#[axum::debug_handler]
pub async fn delete_collection(
    State(repository): State<Arc<dyn CollectionRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository.delete_collection(id, session.profile_id).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            collection::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
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

    #[response(status = 409, description = "Collection already exists")]
    Conflict(BaseError),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum UpdateResponse {
    #[response(status = 200, description = "Updated collection")]
    Ok(Collection),

    #[response(status = 404, description = "Collection not found")]
    NotFound(BaseError),

    #[allow(dead_code)]
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
