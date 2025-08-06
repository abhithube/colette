use std::sync::Arc;

use uuid::Uuid;

use super::{Collection, CollectionCursor, CollectionFindParams, CollectionRepository, Error};
use crate::{
    bookmark::BookmarkFilter,
    collection::{CollectionInsertParams, CollectionUpdateParams},
    pagination::{Paginated, paginate},
};

pub struct CollectionService {
    repository: Arc<dyn CollectionRepository>,
}

impl CollectionService {
    pub fn new(repository: Arc<dyn CollectionRepository>) -> Self {
        Self { repository }
    }

    pub async fn list_collections(
        &self,
        query: CollectionListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Collection, CollectionCursor>, Error> {
        let collections = self
            .repository
            .find(CollectionFindParams {
                user_id: Some(user_id),
                cursor: query.cursor.map(|e| e.title),
                limit: query.limit.map(|e| e + 1),
                ..Default::default()
            })
            .await?;

        if let Some(limit) = query.limit {
            Ok(paginate(collections, limit))
        } else {
            Ok(Paginated {
                items: collections,
                ..Default::default()
            })
        }
    }

    pub async fn get_collection(&self, id: Uuid, user_id: Uuid) -> Result<Collection, Error> {
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
        user_id: Uuid,
    ) -> Result<Collection, Error> {
        let id = self
            .repository
            .insert(CollectionInsertParams {
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
        let Some(collection) = self.repository.find_by_id(id).await? else {
            return Err(Error::NotFound(id));
        };
        if collection.user_id != user_id {
            return Err(Error::Forbidden(id));
        }

        self.repository
            .update(CollectionUpdateParams {
                id,
                title: data.title,
                filter: data.filter,
            })
            .await?;

        self.get_collection(id, user_id).await
    }

    pub async fn delete_collection(&self, id: Uuid, user_id: Uuid) -> Result<(), Error> {
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

#[derive(Debug, Clone, Default)]
pub struct CollectionListQuery {
    pub cursor: Option<CollectionCursor>,
    pub limit: Option<usize>,
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
