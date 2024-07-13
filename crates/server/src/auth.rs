use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use chrono::{DateTime, Utc};
use colette_core::{
    auth::{self, AuthService},
    users,
};

use crate::{
    common,
    common::{BaseError, Context, ValidationError},
    error::Error,
    profiles::Profile,
    session::{Session, SESSION_KEY},
};

#[derive(utoipa::OpenApi)]
#[openapi(paths(register, login), components(schemas(Register, Login, User)))]
pub struct Api;

impl Api {
    pub fn router() -> Router<Context> {
        Router::new().nest(
            "/auth",
            Router::new()
                .route("/register", routing::post(register))
                .route("/login", routing::post(login)),
        )
    }
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = Register,
    responses(RegisterResponse),
    operation_id = "register",
    description = "Register a user account",
    tag = "Auth"
)]
#[axum::debug_handler]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    Valid(Json(body)): Valid<Json<Register>>,
) -> Result<impl IntoResponse, Error> {
    let result = service.register(body.into()).await.map(User::from);

    match result {
        Ok(data) => Ok(RegisterResponse::Created(data)),
        Err(e) => match e {
            auth::Error::Users(users::Error::Conflict(_)) => {
                Ok(RegisterResponse::Conflict(common::BaseError {
                    message: e.to_string(),
                }))
            }
            _ => Err(Error::Unknown),
        },
    }
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = Login,
    responses(LoginResponse),
    operation_id = "login",
    description = "Login to a user account",
    tag = "Auth"
)]
#[axum::debug_handler]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Valid(Json(body)): Valid<Json<Login>>,
) -> Result<impl IntoResponse, Error> {
    let result = service.login(body.into()).await.map(Profile::from);

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
            auth::Error::NotAuthenticated => Ok(LoginResponse::Unauthorized(common::BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    #[schema(format = "email")]
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            email: value.email,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    #[schema(format = "email")]
    #[validate(email(message = "not a valid email"))]
    pub email: String,

    #[schema(min_length = 1)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub password: String,
}

impl From<Register> for auth::Register {
    fn from(value: Register) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    #[schema(format = "email")]
    #[validate(email(message = "not a valid email"))]
    pub email: String,

    #[schema(min_length = 1)]
    #[validate(length(min = 1, message = "cannot be empty"))]
    pub password: String,
}

impl From<Login> for auth::Login {
    fn from(value: Login) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToResponse)]
#[serde(rename_all = "camelCase")]
#[response(description = "Invalid input")]
pub struct RegisterValidationErrors {
    email: Option<Vec<ValidationError>>,
    password: Option<Vec<ValidationError>>,
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum RegisterResponse {
    #[response(status = 201, description = "Registered user")]
    Created(User),

    #[response(status = 409, description = "Email already registered")]
    Conflict(BaseError),

    #[allow(dead_code)]
    #[response(status = 422)]
    UnprocessableEntity(#[to_response] RegisterValidationErrors),
}

impl IntoResponse for RegisterResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(..e)).into_response()
            }
        }
    }
}

#[derive(Debug, serde::Serialize, utoipa::ToResponse)]
#[serde(rename_all = "camelCase")]
#[response(description = "Invalid input")]
pub struct LoginValidationErrors {
    email: Option<Vec<ValidationError>>,
    password: Option<Vec<ValidationError>>,
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum LoginResponse {
    #[response(status = 200, description = "Default profile")]
    Ok(Profile),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(BaseError),

    #[allow(dead_code)]
    #[response(status = 422)]
    UnprocessableEntity(#[to_response] LoginValidationErrors),
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::UnprocessableEntity(e) => {
                (StatusCode::UNPROCESSABLE_ENTITY, Json(e)).into_response()
            }
        }
    }
}
