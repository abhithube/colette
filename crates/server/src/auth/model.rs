use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use colette_core::auth;
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

use crate::{common::Error, profiles::Profile};

#[derive(Debug, Serialize, ToSchema)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct Register {
    #[schema(format = "email")]
    pub email: String,
    #[schema(min_length = 1)]
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

#[derive(Debug, Deserialize, ToSchema)]
pub struct Login {
    #[schema(format = "email")]
    pub email: String,
    #[schema(min_length = 1)]
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

#[derive(Debug, IntoResponses)]
pub enum RegisterResponse {
    #[response(status = 201, description = "Registered user")]
    Created(User),

    #[response(status = 409, description = "Email already registered")]
    Conflict(Error),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(Error),
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

#[derive(Debug, IntoResponses)]
pub enum LoginResponse {
    #[response(status = 200, description = "Default profile")]
    Ok(Profile),

    #[response(status = 401, description = "Bad credentials")]
    Unauthorized(Error),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(Error),
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
