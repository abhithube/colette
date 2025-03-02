use colette_util::base64;
use uuid::Uuid;

use super::{
    BookmarkFilter, Collection, CollectionBookmarkFindParams, Error,
    collection_repository::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
    },
};
use crate::{
    Bookmark, bookmark,
    common::{IdParams, PAGINATION_LIMIT, Paginated},
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
                user_id,
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

    pub async fn list_collection_bookmarks(
        &self,
        id: Uuid,
        query: CollectionBookmarkListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<Bookmark>, Error> {
        let cursor = query.cursor.and_then(|e| base64::decode(&e).ok());

        let collection = self.get_collection(id, user_id).await?;

        let mut bookmarks = self
            .repository
            .find_bookmarks(CollectionBookmarkFindParams {
                filter: collection.filter,
                user_id,
                limit: Some(PAGINATION_LIMIT as i64 + 1),
                cursor,
            })
            .await?;
        let mut cursor: Option<String> = None;

        let limit = PAGINATION_LIMIT as usize;
        if bookmarks.len() > limit {
            bookmarks = bookmarks.into_iter().take(limit).collect();

            if let Some(last) = bookmarks.last() {
                let c = bookmark::Cursor {
                    created_at: last.created_at.unwrap(),
                };
                let encoded = base64::encode(&c)?;

                cursor = Some(encoded);
            }
        }

        Ok(Paginated {
            data: bookmarks,
            cursor,
        })
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

#[derive(Debug, Clone, Default)]
pub struct CollectionBookmarkListQuery {
    pub cursor: Option<String>,
}
