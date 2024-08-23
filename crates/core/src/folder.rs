use uuid::Uuid;

use crate::common::{Creatable, Deletable, Findable, IdParams, Paginated, Updatable};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub collection_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum FolderType {
    #[default]
    All,
    Collections,
    Feeds,
}

#[async_trait::async_trait]
pub trait FolderRepository:
    Findable<Params = IdParams, Output = Result<Folder, Error>>
    + Creatable<Data = FolderCreateData, Output = Result<Folder, Error>>
    + Updatable<Params = IdParams, Data = FolderUpdateData, Output = Result<Folder, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<String>,
        filters: Option<FolderFindManyFilters>,
    ) -> Result<Paginated<Folder>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct FolderFindManyFilters {
    pub folder_type: FolderType,
}

#[derive(Clone, Debug, Default)]
pub struct FolderCreateData {
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug, Default)]
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
