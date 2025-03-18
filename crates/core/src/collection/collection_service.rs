use chrono::Utc;
use uuid::Uuid;

use super::{Collection, CollectionFindParams, CollectionRepository, Error};
use crate::{bookmark::BookmarkFilter, common::Paginated};

pub struct CollectionService {
    repository: Box<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(repository: impl CollectionRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_collections(&self, user_id: String) -> Result<Paginated<Collection>, Error> {
        let collections = self
            .repository
            .find(CollectionFindParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: collections,
            cursor: None,
        })
    }

    pub async fn get_collection(&self, id: Uuid, user_id: String) -> Result<Collection, Error> {
        let mut collections = self
            .repository
            .find(CollectionFindParams {
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
        user_id: String,
    ) -> Result<Collection, Error> {
        let collection = Collection::builder()
            .title(data.title)
            .filter(data.filter)
            .user_id(user_id)
            .build();

        self.repository.save(&collection, false).await?;

        Ok(collection)
    }

    pub async fn update_collection(
        &self,
        id: Uuid,
        data: CollectionUpdate,
        user_id: String,
    ) -> Result<Collection, Error> {
        let Some(mut collection) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if collection.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        if let Some(title) = data.title {
            collection.title = title;
        }
        if let Some(filter) = data.filter {
            collection.filter = filter;
        }

        collection.updated_at = Utc::now();
        self.repository.save(&collection, true).await?;

        Ok(collection)
    }

    pub async fn delete_collection(&self, id: Uuid, user_id: String) -> Result<(), Error> {
        let Some(collection) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if collection.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository.delete_by_id(id).await?;

        Ok(())
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
