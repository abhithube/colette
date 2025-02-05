use colette_netscape::Item;
use colette_opml::Outline;
use uuid::Uuid;

use super::Error;

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error>;

    async fn import_netscape(&self, outlines: Vec<Item>, user_id: Uuid) -> Result<(), Error>;
}
