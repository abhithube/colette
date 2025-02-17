use uuid::Uuid;

use super::{
    CollectionTreeItem, Error, FeedTreeItem, TreeFindParams, library_repository::LibraryRepository,
};
use crate::common::Paginated;

pub struct LibraryService {
    repository: Box<dyn LibraryRepository>,
}

impl LibraryService {
    pub fn new(repository: impl LibraryRepository) -> Self {
        Self {
            repository: Box::new(repository),
        }
    }

    pub async fn list_feed_tree(
        &self,
        query: TreeListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<FeedTreeItem>, Error> {
        let items = self
            .repository
            .find_feed_tree(TreeFindParams {
                folder_id: query.folder_id,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: items,
            cursor: None,
        })
    }

    pub async fn list_collection_tree(
        &self,
        query: TreeListQuery,
        user_id: Uuid,
    ) -> Result<Paginated<CollectionTreeItem>, Error> {
        let items = self
            .repository
            .find_collection_tree(TreeFindParams {
                folder_id: query.folder_id,
                user_id,
                ..Default::default()
            })
            .await?;

        Ok(Paginated {
            data: items,
            cursor: None,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct TreeListQuery {
    pub folder_id: Option<Uuid>,
    pub cursor: Option<String>,
}
