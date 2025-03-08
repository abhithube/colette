use colette_netscape::Item;
use colette_opml::Outline;
use uuid::Uuid;

use super::Error;

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_feeds(&self, params: ImportFeedsParams) -> Result<(), Error>;

    async fn import_bookmarks(&self, params: ImportBookmarksParams) -> Result<(), Error>;
}

pub struct ImportFeedsParams {
    pub outlines: Vec<Outline>,
    pub user_id: Uuid,
}

pub struct ImportBookmarksParams {
    pub items: Vec<Item>,
    pub user_id: Uuid,
}
