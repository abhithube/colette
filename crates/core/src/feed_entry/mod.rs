use chrono::{DateTime, Utc};
use colette_util::base64;
pub use feed_entry_repository::*;
pub use feed_entry_service::*;
use url::Url;
use uuid::Uuid;

use crate::stream;

mod feed_entry_repository;
mod feed_entry_service;

#[derive(Debug, Clone, bon::Builder)]
pub struct FeedEntry {
    #[builder(default = Uuid::new_v4())]
    pub id: Uuid,
    pub link: Url,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<Url>,
    pub feed_id: Uuid,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub published_at: DateTime<Utc>,
    pub id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("feed entry not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access feed entry with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Base64(#[from] base64::Error),

    #[error(transparent)]
    Stream(#[from] stream::Error),

    #[error(transparent)]
    Database(#[from] tokio_postgres::Error),

    #[error(transparent)]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error(transparent)]
    Serde(#[from] serde::de::value::Error),
}
