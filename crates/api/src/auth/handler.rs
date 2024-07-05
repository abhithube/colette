use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use colette_core::{auth::AuthService, common::Session};

use crate::{
    api::SESSION_KEY,
    auth::model::{Login, Register, User},
    error::Error,
    profiles::Profile,
};

#[axum::debug_handler]
#[utoipa::path(
  post,
  path = "/register",
  request_body = Register,
  responses(
    (status = 201, description = "Registered user", body = User)
  ),
  operation_id = "register",
  tag = "Auth"
)]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    Json(body): Json<Register>,
) -> Result<impl IntoResponse, Error> {
    let user = service.register(body.into()).await.map(User::from)?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[axum::debug_handler]
#[utoipa::path(
  post,
  path = "/login",
  request_body = Login,
  responses(
    (status = 200, description = "Active profile", body = Profile)
  ),
  operation_id = "login",
  tag = "Auth"
)]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Json(body): Json<Login>,
) -> Result<impl IntoResponse, Error> {
    let profile = service.login(body.into()).await.map(Profile::from)?;

    let session = Session {
        user_id: profile.user_id.clone(),
        profile_id: profile.id.clone(),
    };
    session_store.insert(SESSION_KEY, session).await?;

    Ok(Json(profile))
}
