use std::sync::Arc;

use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub collection_count: Option<i64>,
    pub feed_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FolderCreate {
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FolderUpdate {
    pub title: Option<NonEmptyString>,
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct FolderListQuery {
    pub folder_type: FolderType,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub enum FolderType {
    #[default]
    All,
    Collections,
    Feeds,
}

pub struct FolderService {
    repository: Arc<dyn FolderRepository>,
}

impl FolderService {
    pub fn new(repository: Arc<dyn FolderRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_folders(
        &self,
        query: FolderListQuery,
        profile_id: Uuid,
    ) -> Result<Paginated<Folder>, Error> {
        self.repository
            .list(profile_id, None, None, Some(query.into()))
            .await
    }

    pub async fn get_folder(&self, id: Uuid, profile_id: Uuid) -> Result<Folder, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_folder(
        &self,
        data: FolderCreate,
        profile_id: Uuid,
    ) -> Result<Folder, Error> {
        self.repository
            .create(FolderCreateData {
                title: data.title.into(),
                parent_id: data.parent_id,
                profile_id,
            })
            .await
    }

    pub async fn update_folder(
        &self,
        id: Uuid,
        data: FolderUpdate,
        profile_id: Uuid,
    ) -> Result<Folder, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete_folder(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }
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

impl From<FolderListQuery> for FolderFindManyFilters {
    fn from(value: FolderListQuery) -> Self {
        Self {
            folder_type: value.folder_type,
        }
    }
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

impl From<FolderUpdate> for FolderUpdateData {
    fn from(value: FolderUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            parent_id: value.parent_id,
        }
    }
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
