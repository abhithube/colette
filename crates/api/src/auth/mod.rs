use std::sync::Arc;

use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use colette_core::auth::AuthService;
use utoipa::OpenApi;
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

use crate::common::{Error, Session, AUTH_TAG};

mod login;
mod logout;
mod register;

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
#[openapi(components(schemas(register::Register, login::Login, User)))]
pub struct AuthApi;

impl AuthApi {
    pub fn router() -> OpenApiRouter<AuthState> {
        OpenApiRouter::with_openapi(AuthApi::openapi())
            .routes(routes!(register::handler))
            .routes(routes!(login::handler))
            .routes(routes!(get_active_user))
            .routes(routes!(logout::handler))
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
) -> Result<GetActiveResponse, Error> {
    match service.get_active(session.user_id).await {
        Ok(data) => Ok(GetActiveResponse::Ok(data.into())),
        Err(e) => Err(Error::Unknown(e.into())),
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
