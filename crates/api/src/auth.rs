use crate::{
    api::{Context, SESSION_KEY},
    error::Error,
};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing, Json, Router};
use colette_core::{
    auth::{AuthService, LoginDto, RegisterDto},
    common::Session,
};
use std::sync::Arc;

pub fn router() -> Router<Context> {
    Router::new()
        .route("/auth/register", routing::post(register))
        .route("/auth/login", routing::post(login))
}

#[axum::debug_handler]
async fn register(
    State(service): State<Arc<AuthService>>,
    Json(body): Json<RegisterDto>,
) -> Result<impl IntoResponse, Error> {
    let user = service.register(body).await?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[axum::debug_handler]
async fn login(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Json(body): Json<LoginDto>,
) -> Result<impl IntoResponse, Error> {
    let profile = service.login(body).await?;

    let session = Session {
        user_id: profile.user_id.clone(),
        profile_id: profile.id.clone(),
    };
    session_store.insert(SESSION_KEY, session).await?;

    Ok(Json(profile))
}
