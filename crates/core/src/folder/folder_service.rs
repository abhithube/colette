use uuid::Uuid;

use super::{
    Error, Folder,
    folder_repository::{FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
};
use crate::common::{IdParams, Paginated};

pub struct FolderService {
    repository: Box<dyn FolderRepository>,
}

impl FolderService {
    pub fn new(repository: impl FolderRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_folders(
        &self,
        query: FolderListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Folder>, Error> {
        let folders = self
            .repository
            .find(FolderFindParams {
                parent_id: query.parent_id,
                user_id,
                ..Default::default()
            })
            .await?;
        Ok(Paginated {
            data: folders,
            cursor: None,
        })
    }

    pub async fn get_folder(&self, id: Uuid, user_id: Uuid) -> Result<Folder, Error> {
        let mut folders = self
            .repository
            .find(FolderFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if folders.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(folders.swap_remove(0))
    }

    pub async fn create_folder(&self, data: FolderCreate, user_id: Uuid) -> Result<Folder, Error> {
        let id = self
            .repository
            .create(FolderCreateData {
                title: data.title,
                parent_id: data.parent_id,
                user_id,
            })
            .await?;

        self.get_folder(id, user_id).await
    }

    pub async fn update_folder(
        &self,
        id: Uuid,
        data: FolderUpdate,
        user_id: Uuid,
    ) -> Result<Folder, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_folder(id, user_id).await
    }

    pub async fn delete_folder(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct FolderListQuery {
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct FolderCreate {
    pub title: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct FolderUpdate {
    pub title: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
}

impl From<FolderUpdate> for FolderUpdateData {
    fn from(value: FolderUpdate) -> Self {
        Self {
            title: value.title,
            parent_id: value.parent_id,
        }
    }
}
