use std::sync::Arc;

use bytes::{Buf, Bytes};
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{Error, backup_repository::BackupRepository};
use crate::{
    Bookmark, Feed, Folder,
    bookmark::{BookmarkFindParams, BookmarkRepository},
    feed::{FeedFindParams, FeedRepository},
    folder::{FolderFindParams, FolderRepository},
    storage::DynStorage,
};

pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    feed_repository: Box<dyn FeedRepository>,
    bookmark_repository: Box<dyn BookmarkRepository>,
    folder_repository: Box<dyn FolderRepository>,
    import_feeds_storage: Arc<Mutex<DynStorage<ImportFeedsJob>>>,
    import_bookmarks_storage: Arc<Mutex<DynStorage<ImportBookmarksJob>>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        feed_repository: impl FeedRepository,
        bookmark_repository: impl BookmarkRepository,
        folder_repository: impl FolderRepository,
        import_feeds_storage: Arc<Mutex<DynStorage<ImportFeedsJob>>>,
        import_bookmarks_storage: Arc<Mutex<DynStorage<ImportBookmarksJob>>>,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            feed_repository: Box::new(feed_repository),
            bookmark_repository: Box::new(bookmark_repository),
            folder_repository: Box::new(folder_repository),
            import_feeds_storage,
            import_bookmarks_storage,
        }
    }

    pub async fn import_opml(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let opml = colette_opml::from_reader(raw.reader())?;

        let urls = opml
            .body
            .outlines
            .iter()
            .filter_map(|e| e.xml_url.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        let mut storage = self.import_feeds_storage.lock().await;

        storage.push(ImportFeedsJob { urls }).await?;

        self.backup_repository
            .import_opml(opml.body.outlines, user_id)
            .await?;

        Ok(())
    }

    pub async fn export_opml(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let folders = self
            .folder_repository
            .find(FolderFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Repository(e.into()))?;

        let feeds = self
            .feed_repository
            .find(FeedFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Repository(e.into()))?;

        let outlines = build_opml_hierarchy(&folders, &feeds, None);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_opml::to_writer(&mut raw, opml)?;

        Ok(raw.into())
    }

    pub async fn import_netscape(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let netscape = colette_netscape::from_reader(raw.reader())?;

        let urls = netscape
            .items
            .iter()
            .filter_map(|e| e.href.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        let mut storage = self.import_bookmarks_storage.lock().await;

        storage.push(ImportBookmarksJob { urls, user_id }).await?;

        self.backup_repository
            .import_netscape(netscape.items, user_id)
            .await?;

        Ok(())
    }

    pub async fn export_netscape(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let folders = self
            .folder_repository
            .find(FolderFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Repository(e.into()))?;

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Repository(e.into()))?;

        let items = build_netscape_hierarchy(&folders, &bookmarks, None);

        let netscape = Netscape {
            items,
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
}

fn build_opml_hierarchy(
    folders: &[Folder],
    feeds: &[Feed],
    parent_id: Option<Uuid>,
) -> Vec<Outline> {
    let mut outlines = Vec::new();

    for folder in folders.iter().filter(|f| f.parent_id == parent_id) {
        let child_outlines = build_opml_hierarchy(folders, feeds, Some(folder.id));

        let outline = Outline {
            text: folder.title.clone(),
            outline: Some(child_outlines),
            ..Default::default()
        };
        outlines.push(outline);
    }

    for feed in feeds.iter().filter(|f| f.folder_id == parent_id) {
        let outline = Outline {
            r#type: Some(OutlineType::default()),
            title: Some(feed.title.clone()),
            text: feed.title.clone(),
            xml_url: feed.xml_url.clone(),
            html_url: Some(feed.link.clone()),
            ..Default::default()
        };

        outlines.push(outline);
    }

    outlines
}

fn build_netscape_hierarchy(
    folders: &[Folder],
    bookmarks: &[Bookmark],
    parent_id: Option<Uuid>,
) -> Vec<Item> {
    let mut items = Vec::new();

    for folder in folders.iter().filter(|f| f.parent_id == parent_id) {
        let child_items = build_netscape_hierarchy(folders, bookmarks, Some(folder.id));

        let item = Item {
            title: folder.title.clone(),
            item: Some(child_items),
            ..Default::default()
        };
        items.push(item);
    }

    for bookmark in bookmarks.iter().filter(|f| f.folder_id == parent_id) {
        let item = Item {
            title: bookmark.title.clone(),
            href: Some(bookmark.link.clone()),
            ..Default::default()
        };

        items.push(item);
    }

    items
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportFeedsJob {
    pub urls: Vec<Url>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ImportBookmarksJob {
    pub urls: Vec<Url>,
    pub user_id: Uuid,
}
