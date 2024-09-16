use std::sync::Arc;

use uuid::Uuid;

use crate::common::{
    Creatable, Deletable, Findable, IdParams, NonEmptyString, Paginated, Updatable,
};

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub bookmark_count: Option<i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct CollectionCreate {
    pub title: NonEmptyString,
    pub parent_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CollectionUpdate {
    pub title: Option<NonEmptyString>,
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Cursor {
    pub title: String,
}

pub struct CollectionService {
    repository: Arc<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(repository: Arc<dyn CollectionRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_collections(&self, profile_id: Uuid) -> Result<Paginated<Collection>, Error> {
        let collections = self.repository.list(profile_id, None, None).await?;

        Ok(Paginated {
            data: collections,
            ..Default::default()
        })
    }

    pub async fn get_collection(&self, id: Uuid, profile_id: Uuid) -> Result<Collection, Error> {
        self.repository.find(IdParams::new(id, profile_id)).await
    }

    pub async fn create_collection(
        &self,
        data: CollectionCreate,
        profile_id: Uuid,
    ) -> Result<Collection, Error> {
        self.repository
            .create(CollectionCreateData {
                title: data.title.into(),
                parent_id: data.parent_id,
                profile_id,
            })
            .await
    }

    pub async fn update_collection(
        &self,
        id: Uuid,
        data: CollectionUpdate,
        profile_id: Uuid,
    ) -> Result<Collection, Error> {
        self.repository
            .update(IdParams::new(id, profile_id), data.into())
            .await
    }

    pub async fn delete_collection(&self, id: Uuid, profile_id: Uuid) -> Result<(), Error> {
        self.repository.delete(IdParams::new(id, profile_id)).await
    }
}

#[async_trait::async_trait]
pub trait CollectionRepository:
    Findable<Params = IdParams, Output = Result<Collection, Error>>
    + Creatable<Data = CollectionCreateData, Output = Result<Collection, Error>>
    + Updatable<Params = IdParams, Data = CollectionUpdateData, Output = Result<Collection, Error>>
    + Deletable<Params = IdParams, Output = Result<(), Error>>
    + Send
    + Sync
{
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
    ) -> Result<Vec<Collection>, Error>;
}

#[derive(Clone, Debug, Default)]
pub struct CollectionCreateData {
    pub title: String,
    pub parent_id: Option<Uuid>,
    pub profile_id: Uuid,
}

#[derive(Clone, Debug, Default)]
pub struct CollectionUpdateData {
    pub title: Option<String>,
    pub parent_id: Option<Option<Uuid>>,
}

impl From<CollectionUpdate> for CollectionUpdateData {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title.map(String::from),
            parent_id: value.parent_id,
        }
    }
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
