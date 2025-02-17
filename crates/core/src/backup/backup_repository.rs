use colette_netscape::Item;
use colette_opml::Outline;
use uuid::Uuid;

use super::Error;
use crate::{Folder, folder::FolderType};

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_feeds(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error>;

    async fn import_bookmarks(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error>;

    async fn export_folders(
        &self,
        folder_type: FolderType,
        user_id: Uuid,
    ) -> Result<Vec<Folder>, Error>;
}
