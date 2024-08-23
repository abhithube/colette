use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use colette_core::{
    common::{IdParams, NonEmptyString},
    folder::{self, FolderCreateData, FolderFindManyFilters, FolderRepository, FolderUpdateData},
};
use uuid::Uuid;

use crate::common::{BaseError, Error, FolderList, Id, Paginated, Session};

#[derive(Clone, axum::extract::FromRef)]
pub struct FolderState {
    repository: Arc<dyn FolderRepository>,
}

impl FolderState {
    pub fn new(repository: Arc<dyn FolderRepository>) -> Self {
        Self { repository }
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(list_folders, get_folder, create_folder, update_folder, delete_folder),
    components(schemas(Folder, FolderList, FolderCreate, FolderUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<FolderState> {
        Router::new().nest(
            "/folders",
            Router::new()
                .route("/", routing::get(list_folders).post(create_folder))
                .route(
                    "/:id",
                    routing::get(get_folder)
                        .patch(update_folder)
                        .delete(delete_folder),
                ),
        )
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
    #[schema(nullable = false)]
    #[serde(skip_serializing_if = "Option::is_none")]
    feed_count: Option<i64>,
}

impl From<colette_core::Folder> for Folder {
    fn from(value: colette_core::Folder) -> Self {
        Self {
            id: value.id,
            title: value.title,
            parent_id: value.parent_id,
            collection_count: value.collection_count,
            feed_count: value.feed_count,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderCreate {
    #[schema(min_length = 1)]
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct FolderUpdate {
    #[schema(min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "serde_with::rust::double_option"
    )]
    pub parent_id: Option<Option<Uuid>>,
}

impl From<FolderUpdate> for FolderUpdateData {
    fn from(value: FolderUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            parent_id: value.parent_id,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct ListFoldersQuery {
    #[param(inline)]
    #[serde(default = "FolderType::default")]
    pub folder_type: FolderType,
}

impl From<ListFoldersQuery> for FolderFindManyFilters {
    fn from(value: ListFoldersQuery) -> Self {
        Self {
            folder_type: value.folder_type.into(),
        }
    }
}

#[derive(Clone, Debug, Default, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum FolderType {
    #[default]
    All,
    Collections,
    Feeds,
}

impl From<FolderType> for folder::FolderType {
    fn from(value: FolderType) -> Self {
        match value {
            FolderType::All => Self::All,
            FolderType::Collections => Self::Collections,
            FolderType::Feeds => Self::Feeds,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    params(ListFoldersQuery),
    responses(ListResponse),
    operation_id = "listFolders",
    description = "List the active profile folders"
)]
#[axum::debug_handler]
pub async fn list_folders(
    State(repository): State<Arc<dyn FolderRepository>>,
    Query(query): Query<ListFoldersQuery>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .list(session.profile_id, None, None, Some(query.into()))
        .await
        .map(Paginated::<Folder>::from);

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
    operation_id = "getFolder",
    description = "Get a folder by ID"
)]
#[axum::debug_handler]
pub async fn get_folder(
    State(repository): State<Arc<dyn FolderRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .find(IdParams::new(id, session.profile_id))
        .await
        .map(Folder::from);

    match result {
        Ok(data) => Ok(GetResponse::Ok(data)),
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
  description = "Create a folder"
)]
#[axum::debug_handler]
pub async fn create_folder(
    State(repository): State<Arc<dyn FolderRepository>>,
    session: Session,
    Json(body): Json<FolderCreate>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .create(FolderCreateData {
            title: body.title.into(),
            parent_id: body.parent_id,
            profile_id: session.profile_id,
        })
        .await
        .map(Folder::from);

    match result {
        Ok(data) => Ok(CreateResponse::Created(data)),
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
    description = "Update a folder by ID"
)]
#[axum::debug_handler]
pub async fn update_folder(
    State(repository): State<Arc<dyn FolderRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<FolderUpdate>,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .update(IdParams::new(id, session.profile_id), body.into())
        .await
        .map(Folder::from);

    match result {
        Ok(data) => Ok(UpdateResponse::Ok(data)),
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
    description = "Delete a folder by ID"
)]
#[axum::debug_handler]
pub async fn delete_folder(
    State(repository): State<Arc<dyn FolderRepository>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = repository
        .delete(IdParams::new(id, session.profile_id))
        .await;

    match result {
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
