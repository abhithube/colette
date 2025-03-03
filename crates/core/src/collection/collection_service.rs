use uuid::Uuid;

use super::{
    Collection, Error,
    collection_repository::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
    },
};
use crate::{
    bookmark::BookmarkFilter,
    common::{IdParams, Paginated},
};

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
            .find_collections(CollectionFindParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: collections,
            cursor: None,
        })
    }

    pub async fn get_collection(&self, id: Uuid, user_id: Uuid) -> Result<Collection, Error> {
        let mut collections = self
            .repository
            .find_collections(CollectionFindParams {
                id: Some(id),
                user_id: Some(user_id),
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
            .create_collection(CollectionCreateData {
                title: data.title,
                filter: data.filter,
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
            .update_collection(IdParams::new(id, user_id), data.into())
            .await?;

        self.get_collection(id, user_id).await
    }

    pub async fn delete_collection(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        self.repository
            .delete_collection(IdParams::new(id, user_id))
            .await
    }
}

impl From<CollectionUpdate> for CollectionUpdateData {
    fn from(value: CollectionUpdate) -> Self {
        Self {
            title: value.title,
            filter: value.filter,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CollectionCreate {
    pub title: String,
    pub filter: BookmarkFilter,
}

#[derive(Debug, Clone, Default)]
pub struct CollectionUpdate {
    pub title: Option<String>,
    pub filter: Option<BookmarkFilter>,
}
