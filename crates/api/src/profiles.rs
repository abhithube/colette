use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing, Json, Router,
};
use colette_core::profiles::{CreateProfileDto, ProfilesService};

use crate::{api::Context, error::Error, session::SessionAuth};

pub fn router() -> Router<Context> {
    Router::new()
        .route("/profiles", routing::get(list).post(create))
        .route("/profiles/@me", routing::get(get_active))
        .route("/profiles/:id", routing::delete(delete))
}

#[axum::debug_handler]
async fn list(
    State(service): State<Arc<ProfilesService>>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let profiles = service.list(session).await?;

    Ok(Json(profiles))
}

#[axum::debug_handler]
async fn get_active(
    State(service): State<Arc<ProfilesService>>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    let profile = service.get(session.profile_id.clone(), session).await?;

    Ok(Json(profile))
}

#[axum::debug_handler]
async fn create(
    State(service): State<Arc<ProfilesService>>,
    SessionAuth(session): SessionAuth,
    Json(body): Json<CreateProfileDto>,
) -> Result<impl IntoResponse, Error> {
    let profile = service.create(body, session).await?;

    Ok((StatusCode::CREATED, Json(profile)))
}

#[axum::debug_handler]
async fn delete(
    State(service): State<Arc<ProfilesService>>,
    Path(id): Path<String>,
    SessionAuth(session): SessionAuth,
) -> Result<impl IntoResponse, Error> {
    service.delete(id, session).await?;

    Ok(StatusCode::NO_CONTENT)
}
