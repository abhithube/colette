use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use colette_core::{
    auth::{self, AuthService},
    users,
};

use super::model::{LoginResponse, RegisterResponse};
use crate::{
    api::{self, SESSION_KEY},
    auth::model::{Login, Register, User},
    error::Error,
    profiles::Profile,
    session::Session,
};

#[axum::debug_handler]
#[utoipa::path(
  post,
  path = "/register",
  request_body = Register,
  responses(RegisterResponse),
  operation_id = "register",
  tag = "Auth"
)]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    Json(body): Json<Register>,
) -> Result<impl IntoResponse, Error> {
    let result = service.register((&body).into()).await.map(User::from);

    match result {
        Ok(data) => Ok(RegisterResponse::Created(data)),
        Err(e) => match e {
            auth::Error::Users(users::Error::Conflict(_)) => {
                Ok(RegisterResponse::Conflict(api::Error {
                    message: e.to_string(),
                }))
            }
            _ => Err(Error::Unknown),
        },
    }
}

#[axum::debug_handler]
#[utoipa::path(
  post,
  path = "/login",
  request_body = Login,
  responses(LoginResponse),
  operation_id = "login",
  tag = "Auth"
)]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Json(body): Json<Login>,
) -> Result<impl IntoResponse, Error> {
    let result = service.login((&body).into()).await.map(Profile::from);

    match result {
        Ok(data) => {
            let session = Session {
                user_id: data.user_id.clone(),
                profile_id: data.id.clone(),
            };
            session_store.insert(SESSION_KEY, session).await?;

            Ok(LoginResponse::Ok(data))
        }
        Err(e) => match e {
            auth::Error::NotAuthenticated => Ok(LoginResponse::Unauthorized(api::Error {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}
