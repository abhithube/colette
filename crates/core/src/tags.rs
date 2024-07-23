use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::common::FindOneParams;

#[derive(Clone, Debug)]
pub struct Tag {
    pub id: Uuid,
    pub title: String,
    pub profile_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[async_trait::async_trait]
pub trait TagsRepository: Send + Sync {
    async fn find_many(&self, params: TagsFindManyParams) -> Result<Vec<Tag>, Error>;

    async fn find_one(&self, params: FindOneParams) -> Result<Tag, Error>;

    async fn create(&self, data: TagsCreateData) -> Result<Tag, Error>;

    async fn update(&self, params: FindOneParams, data: TagsUpdateData) -> Result<Tag, Error>;

    async fn delete(&self, params: FindOneParams) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct TagsFindManyParams {
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct TagsCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct TagsUpdateData {
    pub title: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("tag not found with id: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
