use uuid::Uuid;

use super::{
    Collection, Error,
    collection_repository::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
    },
};
use crate::common::{IdParams, Paginated};

pub struct CollectionService {
    repository: Box<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(repository: impl CollectionRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_collections(
        &self,
        query: CollectionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Collection>, Error> {
        let collections = self
            .repository
            .find(CollectionFindParams {
                folder_id: query.folder_id,
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
                title: data.title,
                folder_id: data.folder_id,
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

#[derive(Debug, Clone, Default)]
pub struct CollectionListQuery {
    pub folder_id: Option<Option<Uuid>>,
}

#[derive(Debug, Clone)]
pub struct CollectionCreate {
    pub title: String,
    pub folder_id: Option<Uuid>,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdate {
    pub title: Option<String>,
    pub folder_id: Option<Option<Uuid>>,
}

impl From<CollectionUpdate> for CollectionUpdateData {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title,
            folder_id: value.folder_id,
        }
    }
}
