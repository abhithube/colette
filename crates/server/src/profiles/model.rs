use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, Utc};
use colette_core::profiles;
use serde::{Deserialize, Serialize};
use url::Url;
use utoipa::{IntoResponses, ToSchema};

use crate::api::{Error, ProfileList};

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub title: String,
    #[schema(format = "uri")]
    pub image_url: Option<String>,
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProfile {
    #[schema(min_length = 1)]
    pub title: String,
    #[schema(nullable = false)]
    pub image_url: Option<Url>,
}

#[derive(Debug, IntoResponses)]
pub enum ListResponse {
    #[response(status = 200, description = "Paginated list of profiles")]
    Ok(ProfileList),
}

#[derive(Debug, IntoResponses)]
pub enum GetActiveResponse {
    #[response(status = 200, description = "Active profile")]
    Ok(Profile),
}

#[derive(Debug, IntoResponses)]
pub enum CreateResponse {
    #[response(status = 201, description = "Created profile")]
    Created(Profile),

    #[allow(dead_code)]
    #[response(status = 422, description = "Invalid input")]
    UnprocessableEntity(Error),
}

#[derive(Debug, IntoResponses)]
pub enum DeleteResponse {
    #[response(status = 204, description = "Successfully deleted profile")]
    NoContent,

    #[response(status = 404, description = "Profile not found")]
    NotFound(Error),

    #[response(status = 409, description = "Deleting default profile")]
    Conflict(Error),
}

impl From<colette_core::Profile> for Profile {
    fn from(value: colette_core::Profile) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<CreateProfile> for profiles::CreateProfile {
    fn from(value: CreateProfile) -> Self {
        Self {
            title: value.title,
            image_url: value.image_url.map(String::from),
        }
    }
}

impl IntoResponse for ListResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

impl IntoResponse for GetActiveResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Ok(data) => Json(data).into_response(),
        }
    }
}

impl IntoResponse for CreateResponse {
    fn into_response(self) -> Response {
        match self {
            Self::Created(data) => (StatusCode::CREATED, Json(data)).into_response(),
            Self::UnprocessableEntity(e) => (StatusCode::UNPROCESSABLE_ENTITY, e).into_response(),
        }
    }
}

impl IntoResponse for DeleteResponse {
    fn into_response(self) -> Response {
        match self {
            Self::NoContent => StatusCode::NO_CONTENT.into_response(),
            Self::NotFound(e) => (StatusCode::NOT_FOUND, e).into_response(),
            Self::Conflict(e) => (StatusCode::CONFLICT, e).into_response(),
        }
    }
}
