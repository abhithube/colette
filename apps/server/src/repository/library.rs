use colette_core::library::{
    CollectionTreeItem, Error, FeedTreeItem, LibraryRepository, TreeFindParams,
};
use sqlx::{Pool, Postgres};

use super::{common, folder::FolderType};

#[derive(Debug, Clone)]
pub struct PostgresLibraryRepository {
    pool: Pool<Postgres>,
}

impl PostgresLibraryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl LibraryRepository for PostgresLibraryRepository {
    async fn find_feed_tree(&self, params: TreeFindParams) -> Result<Vec<FeedTreeItem>, Error> {
        let mut feed_tree_items = common::select_folders(
            &self.pool,
            None,
            params.user_id,
            Some(FolderType::Feeds),
            Some(params.folder_id),
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(FeedTreeItem::Folder).collect::<Vec<_>>())?;

        let mut feeds = common::select_feeds(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            None,
            params.limit,
            None,
        )
        .await
        .map(|e| e.into_iter().map(FeedTreeItem::Feed).collect::<Vec<_>>())?;

        feed_tree_items.append(&mut feeds);

        Ok(feed_tree_items)
    }

    async fn find_collection_tree(
        &self,
        params: TreeFindParams,
    ) -> Result<Vec<CollectionTreeItem>, Error> {
        let mut collection_tree_items = common::select_folders(
            &self.pool,
            None,
            params.user_id,
            Some(FolderType::Collections),
            Some(params.folder_id),
            params.limit,
            None,
        )
        .await
        .map(|e| {
            e.into_iter()
                .map(CollectionTreeItem::Folder)
                .collect::<Vec<_>>()
        })?;

        let mut bookmarks = common::select_collections(
            &self.pool,
            None,
            Some(params.folder_id),
            params.user_id,
            params.limit,
            None,
        )
        .await
        .map(|e| {
            e.into_iter()
                .map(CollectionTreeItem::Collection)
                .collect::<Vec<_>>()
        })?;

        collection_tree_items.append(&mut bookmarks);

        Ok(collection_tree_items)
    }
}
