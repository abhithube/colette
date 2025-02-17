use uuid::Uuid;

use super::{CollectionTreeItem, Error, FeedTreeItem};
use crate::folder::FolderType;

#[async_trait::async_trait]
pub trait LibraryRepository: Send + Sync + 'static {
    async fn find_feed_tree(&self, params: TreeFindParams) -> Result<Vec<FeedTreeItem>, Error>;

    async fn find_collection_tree(
        &self,
        params: TreeFindParams,
    ) -> Result<Vec<CollectionTreeItem>, Error>;
}

#[derive(Debug, Clone)]
pub struct LibraryItemFindParams {
    pub folder_id: Option<Uuid>,
    pub folder_type: FolderType,
    pub user_id: Uuid,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Default)]
pub struct TreeFindParams {
    pub folder_id: Option<Uuid>,
    pub user_id: Uuid,
    pub limit: Option<i64>,
}
