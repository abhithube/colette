use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct CollectionCreate {
    pub title: NonEmptyString,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdate {
    pub title: Option<NonEmptyString>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Cursor {
    pub title: String,
}

pub struct CollectionService {
    repository: Box<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(repository: impl CollectionRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_collections(&self, user_id: Uuid) -> Result<Paginated<Collection>, Error> {
        let collections = self
            .repository
            .find(CollectionFindParams {
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: collections,
            ..Default::default()
        })
    }

    pub async fn get_collection(&self, id: Uuid, user_id: Uuid) -> Result<Collection, Error> {
        let mut collections = self
            .repository
            .find(CollectionFindParams {
                id: Some(id),
                user_id,
                ..Default::default()
            })
            .await?;
        if collections.is_empty() {
            return Err(Error::NotFound(id));
        }

        Ok(collections.swap_remove(0))
    }

    pub async fn create_collection(
        &self,
        data: CollectionCreate,
        user_id: Uuid,
    ) -> Result<Collection, Error> {
        let id = self
            .repository
            .create(CollectionCreateData {
                title: data.title.into(),
                user_id,
            })
            .await?;

        self.get_collection(id, user_id).await
    }

    pub async fn update_collection(
        &self,
        id: Uuid,
        data: CollectionUpdate,
        user_id: Uuid,
    ) -> Result<Collection, Error> {
        self.repository
            .update(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_collection(id, user_id).await
    }

    pub async fn delete_collection(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, user_id)).await
    }
}

pub trait CollectionRepository:
    Findable<Params = CollectionFindParams, Output = Result<Vec<Collection>, Error>>
    + Creatable<Data = CollectionCreateData, Output = Result<Uuid, Error>>
    + Updatable<Params = IdParams, Data = CollectionUpdateData, Output = Result<(), Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
    + 'static
{
}

#[derive(Debug, Clone, Default)]
pub struct CollectionFindParams {
    pub id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<u64>,
    pub cursor: Option<Cursor>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionCreateData {
    pub title: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
}

impl From<CollectionUpdate> for CollectionUpdateData {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("collection not found with ID: {0}")]
    NotFound(Uuid),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}
