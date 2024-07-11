use std::sync::Arc;

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use axum_valid::Valid;
use colette_core::profiles::{self, ProfilesService};

use super::{
    model::{CreateProfile, CreateResponse, DeleteResponse, GetActiveResponse},
    Profile,
};
use crate::{
    common::{self, Id, Paginated},
    error::Error,
    profiles::model::ListResponse,
    session::Session,
};

#[utoipa::path(
    get,
    path = "",
    responses(ListResponse),
    operation_id = "listProfiles",
    description = "List the user profiles",
    tag = "Profiles"
)]
#[axum::debug_handler]
pub async fn list_profiles(
    State(service): State<Arc<ProfilesService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .list(session.into())
        .await
        .map(Paginated::<Profile>::from);

    match result {
        Ok(data) => Ok(ListResponse::Ok(data)),
        Err(_) => Err(Error::Unknown),
    }
}

#[utoipa::path(
    get,
    path = "/@me",
    responses(GetActiveResponse),
    operation_id = "getActiveProfile",
    description = "Get the active profile",
    tag = "Profiles"
)]
#[axum::debug_handler]
pub async fn get_active_profile(
    State(service): State<Arc<ProfilesService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .get(session.profile_id.clone(), session.into())
        .await
        .map(Profile::from);

    match result {
        Ok(data) => Ok(GetActiveResponse::Ok(data)),
        Err(_) => Err(Error::Unknown),
    }
}

#[utoipa::path(
  post,
  path = "",
  request_body = CreateProfile,
  responses(CreateResponse),
  operation_id = "createProfile",
  description = "Create a user profile",
  tag = "Profiles"
)]
#[axum::debug_handler]
pub async fn create_profile(
    State(service): State<Arc<ProfilesService>>,
    session: Session,
    Valid(Json(body)): Valid<Json<CreateProfile>>,
) -> Result<impl IntoResponse, Error> {
    let result = service
        .create(body.into(), session.into())
        .await
        .map(Profile::from);

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
    operation_id = "deleteProfile",
    description = "Delete a profile by ID",
    tag = "Profiles"
)]
#[axum::debug_handler]
pub async fn delete_profile(
    State(service): State<Arc<ProfilesService>>,
    Path(Id(id)): Path<Id>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let result = service.delete(id, session.into()).await;

    match result {
        Ok(()) => Ok(DeleteResponse::NoContent),
        Err(e) => match e {
            profiles::Error::NotFound(_) => Ok(DeleteResponse::NotFound(common::Error {
                message: e.to_string(),
            })),
            profiles::Error::DeletingDefault => Ok(DeleteResponse::Conflict(common::Error {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}
