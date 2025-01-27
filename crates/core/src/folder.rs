use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};
use uuid::Uuid;

#[derive(Clone, Debug, Default, serde::Deserialize)]
pub struct Folder {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug)]
pub struct FolderCreate {
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderUpdate {
    pub title: Option<NonEmptyString>,
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

pub struct FolderService {
    repository: Box<dyn FolderRepository>,
}

impl FolderService {
    pub fn new(repository: impl FolderRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_folders(&self, user_id: Uuid) -> Result<Paginated<Folder>, Error> {
        let folders = self
            .repository
            .find(FolderFindParams {
                user_id,
                ..Default::default()
            })
            .await?;
        Ok(Paginated {
            data: folders,
            ..Default::default()
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
                title: data.title.into(),
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

#[async_trait::async_trait]
pub trait FolderRepository:
    Findable<Params = FolderFindParams, Output = Result<Vec<Folder>, Error>>
    + Creatable<Data = FolderCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = FolderUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Clone, Debug, Default)]
pub struct FolderFindParams {
    pub id: Option<Uuid>,
    pub parent_id: Option<Option<Uuid>>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
    pub cursor: Option<Cursor>,
}

#[derive(Clone, Debug, Default)]
pub struct FolderCreateData {
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub user_id: Uuid,
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
