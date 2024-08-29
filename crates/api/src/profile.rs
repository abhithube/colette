use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use colette_core::{
    common::NonEmptyString,
    profile::{self, ProfileService},
};
use http::StatusCode;
use url::Url;
use uuid::Uuid;

use crate::common::{BaseError, Error, Id, ProfileList, Session};

#[derive(Clone, axum::extract::FromRef)]
pub struct ProfileState {
    service: Arc<ProfileService>,
}

impl ProfileState {
    pub fn new(service: Arc<ProfileService>) -> Self {
        Self { service }
    }
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        list_profiles,
        get_active_profile,
        get_profile,
        create_profile,
        update_profile,
        delete_profile
    ),
    components(schemas(Profile, ProfileList, ProfileCreate, ProfileUpdate))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<ProfileState> {
        Router::new().nest(
            "/profiles",
            Router::new()
                .route("/", routing::get(list_profiles).post(create_profile))
                .route("/@me", routing::get(get_active_profile))
                .route(
                    "/:id",
                    routing::get(get_profile)
                        .patch(update_profile)
                        .delete(delete_profile),
                ),
        )
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: Uuid,
    pub title: String,
    #[schema(format = "uri", required)]
    pub image_url: Option<String>,
    pub is_default: bool,
    pub user_id: Uuid,
}

impl From<colette_core::Profile> for Profile {
    fn from(value: colette_core::Profile) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            is_default: value.is_default,
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileCreate {
    #[schema(value_type = String, min_length = 1)]
    pub title: NonEmptyString,
    pub image_url: Option<Url>,
}

impl From<ProfileCreate> for profile::ProfileCreate {
    fn from(value: ProfileCreate) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdate {
    #[schema(value_type = Option<String>, min_length = 1, nullable = false)]
    pub title: Option<NonEmptyString>,
    pub image_url: Option<Url>,
}

impl From<ProfileUpdate> for profile::ProfileUpdate {
    fn from(value: ProfileUpdate) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url,
        }
    }
}

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listProfiles",
    description = "List the user profiles"
)]
#[axum::debug_handler]
pub async fn list_profiles(
    State(service): State<Arc<ProfileService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.list_profiles(session.user_id).await {
        Ok(data) => Ok(ListResponse::Ok(data.into())),
        Err(_) => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    params(Id),
    responses(GetResponse),
    operation_id = "getProfile",
    description = "Get a profile by ID"
)]
#[axum::debug_handler]
pub async fn get_profile(
    State(service): State<Arc<ProfileService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_profile(id, session.user_id).await {
        Ok(data) => Ok(GetResponse::Ok(data.into())),
        Err(e) => match e {
            profile::Error::NotFound(_) => Ok(GetResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    get,
    path = "/@me",
    responses(GetActiveResponse),
    operation_id = "getActiveProfile",
    description = "Get the active profile"
)]
#[axum::debug_handler]
pub async fn get_active_profile(
    State(service): State<Arc<ProfileService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_active_profile(session.user_id).await {
        Ok(data) => Ok(GetActiveResponse::Ok(data.into())),
        Err(_) => Err(Error::Unknown),
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = ProfileCreate,
  responses(CreateResponse),
  operation_id = "createProfile",
  description = "Create a user profile"
)]
#[axum::debug_handler]
pub async fn create_profile(
    State(service): State<Arc<ProfileService>>,
    session: Session,
    Json(body): Json<ProfileCreate>,
) -> Result<impl IntoResponse, Error> {
    match service.create_profile(body.into(), session.user_id).await {
        Ok(data) => Ok(CreateResponse::Created(data.into())),
        Err(e) => match e {
            profile::Error::Conflict(_) => Ok(CreateResponse::Conflict(BaseError {
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
    request_body = ProfileUpdate,
    responses(UpdateResponse),
    operation_id = "updateProfile",
    description = "Update a profile by ID"
)]
#[axum::debug_handler]
pub async fn update_profile(
    State(service): State<Arc<ProfileService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
    Json(body): Json<ProfileUpdate>,
) -> Result<impl IntoResponse, Error> {
    match service
        .update_profile(id, body.into(), session.user_id)
        .await
    {
        Ok(data) => Ok(UpdateResponse::Ok(data.into())),
        Err(e) => match e {
            profile::Error::NotFound(_) => Ok(UpdateResponse::NotFound(BaseError {
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
    operation_id = "deleteProfile",
    description = "Delete a profile by ID"
)]
#[axum::debug_handler]
pub async fn delete_profile(
    State(service): State<Arc<ProfileService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.delete_profile(id, session.user_id).await {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            profile::Error::NotFound(_) => Ok(DeleteResponse::NotFound(BaseError {
                message: e.to_string(),
            })),
            profile::Error::DeletingDefault => Ok(DeleteResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of profiles")]
    Ok(ProfileList),
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
    #[response(status = 200, description = "Profile by ID")]
    Ok(Profile),

    #[response(status = 404, description = "Profile not found")]
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
pub enum GetActiveResponse {
    #[response(status = 200, description = "Active profile")]
    Ok(Profile),
}

impl IntoResponse for GetActiveResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created profile")]
    Created(Profile),

    #[response(status = 409, description = "Profile already exists")]
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
    #[response(status = 200, description = "Updated profile")]
    Ok(Profile),

    #[response(status = 404, description = "Profile not found")]
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
    #[response(status = 204, description = "Successfully deleted profile")]
    NoContent,

    #[response(status = 404, description = "Profile not found")]
    NotFound(BaseError),

    #[response(status = 409, description = "Deleting default profile")]
    Conflict(BaseError),
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
        }
    }
}
