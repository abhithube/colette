use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::{
    auth::{self, AuthService},
    common::NonEmptyString,
    user,
};
use email_address::EmailAddress;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Session, AUTH_TAG, SESSION_KEY},
    profile::Profile,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct AuthState {
    auth_service: Arc<AuthService>,
}

impl AuthState {
    pub fn new(auth_service: Arc<AuthService>) -> Self {
        Self { auth_service }
    }
}

#[derive(OpenApi)]
#[openapi(components(schemas(Register, Login, User, SwitchProfile)))]
pub struct AuthApi;

impl AuthApi {
    pub fn router() -> OpenApiRouter<AuthState> {
        OpenApiRouter::with_openapi(AuthApi::openapi())
            .routes(routes!(register))
            .routes(routes!(login))
            .routes(routes!(get_active_user))
            .routes(routes!(switch_profile))
            .routes(routes!(logout))
    }
}

#[derive(Clone, Debug, serde::Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    #[schema(format = "email")]
    pub email: String,
}

impl From<colette_core::User> for User {
    fn from(value: colette_core::User) -> Self {
        Self {
            id: value.id,
            email: value.email,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Register {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
}

impl From<Register> for auth::Register {
    fn from(value: Register) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Login {
    #[schema(value_type = String, format = "email")]
    pub email: EmailAddress,
    #[schema(value_type = String, min_length = 1)]
    pub password: NonEmptyString,
}

impl From<Login> for auth::Login {
    fn from(value: Login) -> Self {
        Self {
            email: value.email,
            password: value.password,
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SwitchProfile {
    pub id: Uuid,
}

impl From<SwitchProfile> for auth::SwitchProfile {
    fn from(value: SwitchProfile) -> Self {
        Self { id: value.id }
    }
}

#[utoipa::path(
    post,
    path = "/register",
    request_body = Register,
    responses(RegisterResponse),
    operation_id = "register",
    description = "Register a user account",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn register(
    State(service): State<Arc<AuthService>>,
    Json(body): Json<Register>,
) -> Result<impl IntoResponse, Error> {
    match service.register(body.into()).await {
        Ok(data) => Ok(RegisterResponse::Created(data.into())),
        Err(e) => match e {
            auth::Error::Users(user::Error::Conflict(_)) => {
                Ok(RegisterResponse::Conflict(BaseError {
                    message: e.to_string(),
                }))
            }
            e => Err(Error::Unknown(e.into())),
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
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn login(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    Json(body): Json<Login>,
) -> Result<impl IntoResponse, Error> {
    match service.login(body.into()).await {
        Ok(data) => {
            let session = Session {
                user_id: data.user_id,
                profile_id: data.id,
            };
            session_store.insert(SESSION_KEY, session).await?;

            Ok(LoginResponse::Ok(data.into()))
        }
        Err(e) => match e {
            auth::Error::NotAuthenticated => Ok(LoginResponse::Unauthorized(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    get,
    path = "/@me",
    responses(GetActiveResponse),
    operation_id = "getActiveUser",
    description = "Get the active user",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn get_active_user(
    State(service): State<Arc<AuthService>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    match service.get_active(session.user_id).await {
        Ok(data) => Ok(GetActiveResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
    }
}

#[utoipa::path(
    post,
    path = "/switchProfile",
    request_body = SwitchProfile,
    responses(SwitchProfileResponse),
    operation_id = "switchProfile",
    description = "Switch to a different profile",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn switch_profile(
    State(service): State<Arc<AuthService>>,
    session_store: tower_sessions::Session,
    session: Session,
    Json(body): Json<SwitchProfile>,
) -> Result<impl IntoResponse, Error> {
    match service.switch_profile(body.into(), session.user_id).await {
        Ok(data) => {
            let session = Session {
                user_id: data.user_id,
                profile_id: data.id,
            };
            session_store.insert(SESSION_KEY, session).await?;

            Ok(SwitchProfileResponse::Ok(data.into()))
        }
        Err(e) => match e {
            auth::Error::NotAuthenticated => Ok(SwitchProfileResponse::Unauthorized(BaseError {
                message: e.to_string(),
            })),
            e => Err(Error::Unknown(e.into())),
        },
    }
}

#[utoipa::path(
    post,
    path = "/logout",
    responses(LogoutResponse),
    operation_id = "logout",
    description = "Log out of user account",
    tag = AUTH_TAG
)]
#[axum::debug_handler]
pub async fn logout(session_store: tower_sessions::Session) -> Result<impl IntoResponse, Error> {
    session_store.delete().await?;

    Ok(LogoutResponse::NoContent)
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum RegisterResponse {
    #[response(status = 201, description = "Registered user")]
    Created(User),

    #[response(status = 409, description = "Email already registered")]
    Conflict(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for RegisterResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum LoginResponse {
    #[response(status = 200, description = "Default profile")]
    Ok(Profile),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for LoginResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum GetActiveResponse {
    #[response(status = 200, description = "Active user")]
    Ok(User),
}

impl IntoResponse for GetActiveResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum SwitchProfileResponse {
    #[response(status = 200, description = "Selected profile")]
    Ok(Profile),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(BaseError),

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for SwitchProfileResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
            Self::Unauthorized(e) => (StatusCode::UNAUTHORIZED, e).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

#[derive(Debug, utoipa::IntoResponses)]
pub enum LogoutResponse {
    #[response(status = 204, description = "Successfully logged out")]
    NoContent,

    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(BaseError),
}

impl IntoResponse for LogoutResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}
