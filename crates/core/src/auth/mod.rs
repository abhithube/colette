pub use auth_service::*;
use chrono::{DateTime, Utc};
use url::Url;
pub use user_repository::*;
use uuid::Uuid;

mod auth_service;
mod user_repository;

#[derive(Debug, Clone, serde::Deserialize, bon::Builder)]
pub struct User {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub external_id: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub picture_url: Option<Url>,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NotFound(NotFoundError),

    #[error("Missing JWT key ID")]
    MissingKid,

    #[error("Missing JWK")]
    MissingJwk,

    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error(transparent)]
    Http(#[from] colette_http::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),
}

#[derive(Debug, thiserror::Error)]
pub enum NotFoundError {
    #[error("user not found with ID: {0}")]
    Id(Uuid),

    #[error("user not found with external ID: {0}")]
    ExternalId(String),
}
