use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing, Json, Router};
use colette_core::auth::{AuthService, LoginDto, RegisterDto};

use crate::{api::Context, error::Error};

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
    Json(body): Json<LoginDto>,
) -> Result<impl IntoResponse, Error> {
    let user = service.login(body).await?;

    Ok(Json(user))
}
