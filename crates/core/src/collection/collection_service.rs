use uuid::Uuid;

use super::{
    Collection, CollectionCreateData, CollectionFindParams, CollectionRepository,
    CollectionUpdateData, Error,
};
use crate::{
    bookmark::BookmarkFilter,
    common::{Paginated, TransactionManager},
};

pub struct CollectionService {
    repository: Box<dyn CollectionRepository>,
    tx_manager: Box<dyn TransactionManager>,
}

impl CollectionService {
    pub fn new(repository: impl CollectionRepository, tx_manager: impl TransactionManager) -> Self {
        Self {
            repository: Box::new(repository),
            tx_manager: Box::new(tx_manager),
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
                ..Default::default()
            })
            .await?;
        if collections.is_empty() {
            return Err(Error::NotFound(id));
        }

        let collection = collections.swap_remove(0);
        if collection.user_id != user_id {
            return Err(Error::Forbidden(collection.id));
        }

        Ok(collection)
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
        let tx = self.tx_manager.begin().await?;

        let collection = self.repository.find_collection_by_id(&*tx, id).await?;
        if collection.user_id != user_id {
            return Err(Error::Forbidden(collection.id));
        }

        self.repository
            .update_collection(&*tx, collection.id, data.into())
            .await?;

        tx.commit().await?;

        self.get_collection(collection.id, collection.user_id).await
    }

    pub async fn delete_collection(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
        let tx = self.tx_manager.begin().await?;

        let collection = self.repository.find_collection_by_id(&*tx, id).await?;
        if collection.user_id != user_id {
            return Err(Error::Forbidden(collection.id));
        }

        self.repository
            .delete_collection(&*tx, collection.id)
            .await?;

        tx.commit().await?;

        Ok(())
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
