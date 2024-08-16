use uuid::Uuid;

use crate::common::Paginated;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub bookmark_count: Option<i64>,
}

#[async_trait::async_trait]
pub trait CollectionsRepository: Send + Sync {
    async fn find_many_collections(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
    ) -> Result<Paginated<Collection>, Error>;

    async fn find_one_collection(&self, id: Uuid, profile_id: Uuid) -> Result<Collection, Error>;

    async fn create_collection(&self, data: CollectionsCreateData) -> Result<Collection, Error>;

    async fn update_collection(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: CollectionsUpdateData,
    ) -> Result<Collection, Error>;

    async fn delete_collection(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct CollectionsCreateData {
    pub title: String,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct CollectionsUpdateData {
    pub title: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error("collection already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
