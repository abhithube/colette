use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{
    common::NonEmptyString,
    folder::{self, FolderService},
};
use http::StatusCode;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::common::{BaseError, Error, FolderList, Id, Session, FOLDERS_TAG};

#[derive(Clone, axum::extract::FromRef)]
pub struct FolderState {
    service: Arc<FolderService>,
}

impl FolderState {
    pub fn new(service: Arc<FolderService>) -> Self {
        Self { service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Folder, FolderList, FolderCreate, FolderUpdate)))]
pub struct FolderApi;

impl FolderApi {
    pub fn router() -> OpenApiRouter<FolderState> {
        OpenApiRouter::with_openapi(FolderApi::openapi())
            .routes(routes!(list_folders, create_folder))
            .routes(routes!(get_folder, update_folder, delete_folder))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    #[schema(nullable = false)]
    pub parent_id: Option<Uuid>,
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    collection_count: Option<i64>,
}

impl From<colette_core::Folder> for Folder {
    fn from(value: colette_core::Folder) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            collection_count: value.collection_count,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

impl From<FolderCreate> for folder::FolderCreate {
    fn from(value: FolderCreate) -> Self {
        Self {
            title: value.title,
            parent_id: value.parent_id,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub parent_id: Option<Option<Uuid>>,
}

impl From<FolderUpdate> for folder::FolderUpdate {
    fn from(value: FolderUpdate) -> Self {
        Self {
            title: value.title,
            parent_id: value.parent_id,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listFolders",
    description = "List the active profile folders",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn list_folders(
    State(service): State<Arc<FolderService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.list_folders(session.profile_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        _ => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getFolder",
    description = "Get a folder by ID",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn get_folder(
    State(service): State<Arc<FolderService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_folder(id, session.profile_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            folder::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = FolderCreate,
  responses(CreateResponse),
  operation_id = "createFolder",
  description = "Create a folder",
  tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn create_folder(
    State(service): State<Arc<FolderService>>,
    session: Session,
    Json(body): Json<FolderCreate>,
) -> Result<impl IntoResponse, Error> {
    match service.create_folder(body.into(), session.profile_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            folder::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
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
    request_body = FolderUpdate,
    responses(UpdateResponse),
    operation_id = "updateFolder",
    description = "Update a folder by ID",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn update_folder(
    State(service): State<Arc<FolderService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FolderUpdate>,
) -> Result<impl IntoResponse, Error> {
    match service
        .update_folder(id, body.into(), session.profile_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            folder::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    operation_id = "deleteFolder",
    description = "Delete a folder by ID",
    tag = FOLDERS_TAG
)]
#[axum::debug_handler]
pub async fn delete_folder(
    State(service): State<Arc<FolderService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.delete_folder(id, session.profile_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            folder::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of folders")]
    Ok(FolderList),
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
    #[response(status = 200, description = "Folder by ID")]
    Ok(Folder),

    #[response(status = 404, description = "Folder not found")]
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
    #[response(status = 201, description = "Created folder")]
    Created(Folder),

    #[response(status = 409, description = "Folder already exists")]
    Conflict(BaseError),

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
    #[response(status = 200, description = "Updated folder")]
    Ok(Folder),

    #[response(status = 404, description = "Folder not found")]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted folder")]
    NoContent,

    #[response(status = 404, description = "Folder not found")]
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
