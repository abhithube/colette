use colette_netscape::Item;
use uuid::Uuid;

use super::Error;

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_feeds(
        &self,
        outlines: Vec<colette_opml::Outline>,
        user_id: Uuid,
    ) -> Result<(), Error>;

    async fn import_bookmarks(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error>;

    async fn export_outlines(&self, user_id: Uuid) -> Result<Vec<super::Outline>, Error>;

    async fn export_items(&self, user_id: Uuid) -> Result<Vec<super::Item>, Error>;
}
