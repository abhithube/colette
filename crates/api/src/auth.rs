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
    profile::{ProfileIdOrDefaultParams, ProfileRepository},
    user::{self, UserCreateData, UserIdParams, UserRepository},
};
use colette_utils::password;
use uuid::Uuid;

use crate::{
    common::{BaseError, Error, Session, SESSION_KEY},
    profile::Profile,
};

#[derive(Clone, axum::extract::FromRef)]
pub struct AuthState {
    pub user_repository: Arc<dyn UserRepository>,
    pub profile_repository: Arc<dyn ProfileRepository>,
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

#[utoipa::path(
    post,
    path = "/register",
    request_body = Register,
    responses(RegisterResponse),
    operation_id = "register",
    description = "Register a user account"
)]
#[axum::debug_handler]
pub async fn register(
    State(repository): State<Arc<dyn UserRepository>>,
    Valid(Json(body)): Valid<Json<Register>>,
) -> Result<impl IntoResponse, Error> {
    let hashed = password::hash(&body.password)
        .await
        .map_err(|_| Error::Unknown)?;

    let result = repository
        .create(UserCreateData {
            email: body.email,
            password: hashed,
        })
        .await
        .map(User::from);

    match result {
        Ok(data) => Ok(RegisterResponse::Created(data)),
        Err(e) => match e {
            user::Error::Conflict(_) => Ok(RegisterResponse::Conflict(BaseError {
                message: e.to_string(),
            })),
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
    description = "Login to a user account"
)]
#[axum::debug_handler]
pub async fn login(
    State(AuthState {
        user_repository,
        profile_repository,
    }): State<AuthState>,
    session_store: tower_sessions::Session,
    Valid(Json(body)): Valid<Json<Login>>,
) -> Result<impl IntoResponse, Error> {
    let result = user_repository.find(UserIdParams::Email(body.email)).await;

    if let Err(e) = result {
        match e {
            user::Error::NotFound(_) => {
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

    let Ok(valid) = password::verify(&body.password, &user.password).await else {
        return Err(Error::Unknown);
    };
    if !valid {
        return Ok(LoginResponse::Unauthorized(BaseError {
            message: "bad credentials".to_owned(),
        }));
    }

    let result = profile_repository
        .find(ProfileIdOrDefaultParams {
            id: None,
            user_id: user.id,
        })
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

#[utoipa::path(
    get,
    path = "/@me",
    responses(GetActiveResponse),
    operation_id = "getActiveUser",
    description = "Get the active user"
)]
#[axum::debug_handler]
pub async fn get_active_user(
    State(repository): State<Arc<dyn UserRepository>>,
    session: Session,
) -> Result<impl IntoResponse, Error> {
    let user = repository
        .find(UserIdParams::Id(session.user_id))
        .await
        .map(User::from)
        .map_err(|_| Error::Unknown)?;

    Ok(GetActiveResponse::Ok(user))
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
