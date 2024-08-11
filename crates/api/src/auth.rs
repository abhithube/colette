use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing, Json, Router,
};
use axum_valid::Valid;
use colette_core::{
    auth,
    profiles::ProfilesRepository,
    users::{self, UsersCreateData, UsersFindOneParams, UsersRepository},
    utils::password::PasswordHasher,
};
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Session, SESSION_KEY},
    profiles::Profile,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct AuthState {
    pub users_repository: Arc<dyn UsersRepository>,
    pub profiles_repository: Arc<dyn ProfilesRepository>,
    pub hasher: Arc<dyn PasswordHasher>,
}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(register, login, get_active_user),
    components(schemas(Register, Login, User))
)]
pub struct Api;

impl Api {
    pub fn router() -> Router<AuthState> {
        Router::new().nest(
            "/auth",
            Router::new()
                .route("/register", routing::post(register))
                .route("/login", routing::post(login))
                .route("/@me", routing::get(get_active_user)),
        )
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
    State(AuthState {
        users_repository,
        hasher,
        ..
    }): State<AuthState>,
    Valid(Json(body)): Valid<Json<Register>>,
) -> Result<impl IntoResponse, Error> {
    let hashed = hasher
        .hash(&body.password)
        .await
        .map_err(|_| Error::Unknown)?;

    let result = users_repository
        .create_user(UsersCreateData {
            email: body.email,
            password: hashed,
        })
        .await
        .map(User::from)
        .map_err(|e| e.into());

    match result {
        Ok(data) => Ok(RegisterResponse::Created(data)),
        Err(e) => match e {
            auth::Error::Users(users::Error::Conflict(_)) => {
                Ok(RegisterResponse::Conflict(BaseError {
                    message: e.to_string(),
                }))
            }
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum RegisterResponse {
    #[response(status = 201, description = "Registered user")]
    Created(User),

    #[response(status = 409, description = "Email already registered")]
    Conflict(BaseError),

    #[allow(dead_code)]
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
    State(AuthState {
        users_repository,
        profiles_repository,
        hasher,
        ..
    }): State<AuthState>,
    session_store: tower_sessions::Session,
    Valid(Json(body)): Valid<Json<Login>>,
) -> Result<impl IntoResponse, Error> {
    let result = users_repository
        .find_one_user(UsersFindOneParams::Email(body.email))
        .await;

    if let Err(e) = result {
        match e {
            users::Error::NotFound(_) => {
                return Ok(LoginResponse::Unauthorized(BaseError {
                    message: "bad credentials".to_owned(),
                }));
            }
            _ => {
                return Err(Error::Unknown);
            }
        }
    };
    let user = result.unwrap();

    let valid = hasher
        .verify(&body.password, &user.password)
        .await
        .map_err(|_| Error::Unknown)?;
    if !valid {
        return Ok(LoginResponse::Unauthorized(BaseError {
            message: "bad credentials".to_owned(),
        }));
    }

    let result = profiles_repository
        .find_one_profile(None, user.id)
        .await
        .map(Profile::from)
        .map_err(|e| e.into());

    match result {
        Ok(data) => {
            let session = Session {
                user_id: data.user_id,
                profile_id: data.id,
            };
            session_store.insert(SESSION_KEY, session).await?;

            Ok(LoginResponse::Ok(data))
        }
        Err(e) => match e {
            auth::Error::NotAuthenticated => Ok(LoginResponse::Unauthorized(BaseError {
                message: e.to_string(),
            })),
            _ => Err(Error::Unknown),
        },
    }
}

#[derive(Clone, Debug, serde::Deserialize, utoipa::ToSchema, validator::Validate)]
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

#[derive(Debug, utoipa::IntoResponses)]
pub enum LoginResponse {
    #[response(status = 200, description = "Default profile")]
    Ok(Profile),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(BaseError),

    #[allow(dead_code)]
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

#[utoipa::path(
    get,
    path = "/@me",
    responses(GetActiveResponse),
    operation_id = "getActiveUser",
    description = "Get the active user",
    tag = "Auth"
)]
#[axum::debug_handler]
pub async fn get_active_user(
    State(repository): State<Arc<dyn UsersRepository>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let user = repository
        .find_one_user(UsersFindOneParams::Id(session.user_id))
        .await
        .map(User::from)
        .map_err(|_| Error::Unknown)?;

    Ok(GetActiveResponse::Ok(user))
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
