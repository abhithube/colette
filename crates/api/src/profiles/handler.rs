use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use colette_core::profiles::ProfilesService;

use super::{model::CreateProfileDto, ProfileDto};
use crate::{api::Paginated, error::Error, session::SessionDto};

#[axum::debug_handler]
#[utoipa::path(
  get,
  path = "",
  responses(
    (status = 200, description = "Paginated list of profiles", body = ProfileList)
  ),
  operation_id = "listProfiles",
  tag = "Profile"
)]
pub async fn list_profiles(
    State(service): State<Arc<ProfilesService>>,
    session: SessionDto,
) -> Result<impl IntoResponse, Error> {
    let profiles = service
        .list((&session).into())
        .await
        .map(Paginated::<ProfileDto>::from)?;

    Ok(Json(profiles))
}

#[utoipa::path(
  get,
  path = "/@me",
  responses(
    (status = 200, description = "Active profile", body = Profile)
  ),
  operation_id = "getActiveProfile",
  tag = "Profile"
)]
#[axum::debug_handler]
pub async fn get_active_profile(
    State(service): State<Arc<ProfilesService>>,
    session: SessionDto,
) -> Result<impl IntoResponse, Error> {
    let profile = service
        .get(session.profile_id.clone(), (&session).into())
        .await
        .map(ProfileDto::from)?;

    Ok(Json(profile))
}

#[utoipa::path(
  post,
  path = "",
  request_body = CreateProfile,
  responses(
    (status = 201, description = "Created profile", body = Profile)
  ),
  operation_id = "createProfile",
  tag = "Profile"
)]
#[axum::debug_handler]
pub async fn create_profile(
    State(service): State<Arc<ProfilesService>>,
    session: SessionDto,
    Json(body): Json<CreateProfileDto>,
) -> Result<impl IntoResponse, Error> {
    let profile = service
        .create((&body).into(), (&session).into())
        .await
        .map(ProfileDto::from)?;

    Ok((StatusCode::CREATED, Json(profile)))
}

#[utoipa::path(
  delete,
  path = "/{id}",
  params(
    ("id", description = "Profile ID")
  ),
  responses(
    (status = 204, description = "Successfully deleted profile")
  ),
  operation_id = "deleteProfile",
  tag = "Profile"
)]
#[axum::debug_handler]
pub async fn delete_profile(
    State(service): State<Arc<ProfilesService>>,
    Path(id): Path<String>,
    session: SessionDto,
) -> Result<impl IntoResponse, Error> {
    service.delete(id, (&session).into()).await?;

    Ok(StatusCode::NO_CONTENT)
}
