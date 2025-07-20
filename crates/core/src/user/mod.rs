use chrono::{DateTime, Utc};
use url::Url;
pub use user_repository::*;
use uuid::Uuid;

mod user_repository;

#[derive(Debug, Clone, serde::Deserialize, bon::Builder)]
pub struct User {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub image_url: Option<Url>,
    #[builder(default = Utc::now())]
    pub created_at: DateTime<Utc>,
    #[builder(default = Utc::now())]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
