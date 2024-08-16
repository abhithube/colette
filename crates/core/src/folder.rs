use uuid::Uuid;

use crate::common::Paginated;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub collection_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum FolderType {
    All,
    Collections,
    Feeds,
}

#[async_trait::async_trait]
pub trait FolderRepository: Send + Sync {
    async fn find_many_folders(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<FolderFindManyFilters>,
    ) -> Result<Paginated<Folder>, Error>;

    async fn find_one_folder(&self, id: Uuid, profile_id: Uuid) -> Result<Folder, Error>;

    async fn create_folder(&self, data: FolderCreateData) -> Result<Folder, Error>;

    async fn update_folder(
        &self,
        id: Uuid,
        profile_id: Uuid,
        data: FolderUpdateData,
    ) -> Result<Folder, Error>;

    async fn delete_folder(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error>;
}

#[derive(Clone, Debug)]
pub struct FolderFindManyFilters {
    pub folder_type: FolderType,
}

#[derive(Clone, Debug)]
pub struct FolderCreateData {
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug)]
pub struct FolderUpdateData {
    pub title: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("folder not found with ID: {0}")]
    NotFound(Uuid),

    #[error("folder already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
