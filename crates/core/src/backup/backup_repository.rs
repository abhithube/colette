use colette_netscape::Item;
use colette_opml::Outline;

use super::Error;

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_feeds(&self, data: ImportFeedsData) -> Result<(), Error>;

    async fn import_bookmarks(&self, data: ImportBookmarksData) -> Result<(), Error>;
}

pub struct ImportFeedsData {
    pub outlines: Vec<Outline>,
    pub user_id: String,
}

pub struct ImportBookmarksData {
    pub items: Vec<Item>,
    pub user_id: String,
}
