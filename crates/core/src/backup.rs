use bytes::{buf::Reader, Buf, Bytes};
use colette_backup::BackupManager;
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use url::Url;
use uuid::Uuid;

use crate::{
    bookmark::{BookmarkFindParams, BookmarkRepository},
    feed::{FeedFindParams, FeedRepository},
    folder::{FolderFindParams, FolderRepository},
    Bookmark, Feed, Folder,
};

pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    feed_repository: Box<dyn FeedRepository>,
    bookmark_repository: Box<dyn BookmarkRepository>,
    folder_repository: Box<dyn FolderRepository>,
    opml_manager: Box<dyn BackupManager<Reader<Bytes>, Data = Opml>>,
    netscape_manager: Box<dyn BackupManager<Reader<Bytes>, Data = Netscape>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        feed_repository: impl FeedRepository,
        bookmark_repository: impl BookmarkRepository,
        folder_repository: impl FolderRepository,
        opml_manager: impl BackupManager<Reader<Bytes>, Data = Opml>,
        netscape_manager: impl BackupManager<Reader<Bytes>, Data = Netscape>,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            feed_repository: Box::new(feed_repository),
            bookmark_repository: Box::new(bookmark_repository),
            folder_repository: Box::new(folder_repository),
            opml_manager: Box::new(opml_manager),
            netscape_manager: Box::new(netscape_manager),
        }
    }

    pub async fn import_opml(&self, raw: Bytes, user_id: Uuid) -> Result<Vec<Url>, Error> {
        let opml = self
            .opml_manager
            .import(raw.reader())
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let urls = opml
            .body
            .outlines
            .iter()
            .filter_map(|e| e.xml_url.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        self.backup_repository
            .import_opml(opml.body.outlines, user_id)
            .await?;

        Ok(urls)
    }

    pub async fn export_opml(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let folders = self
            .folder_repository
            .find(FolderFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let feeds = self
            .feed_repository
            .find(FeedFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let outlines = build_opml_hierarchy(&folders, &feeds, None);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        self.opml_manager
            .export(&mut raw, opml)
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        Ok(raw.into())
    }

    pub async fn import_netscape(&self, raw: Bytes, user_id: Uuid) -> Result<Vec<Url>, Error> {
        let netscape = self
            .netscape_manager
            .import(raw.reader())
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        let urls = netscape
            .items
            .iter()
            .filter_map(|e| e.href.as_deref().and_then(|e| Url::parse(e).ok()))
            .collect::<Vec<Url>>();

        self.backup_repository
            .import_netscape(netscape.items, user_id)
            .await?;

        Ok(urls)
    }

    pub async fn export_netscape(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let folders = self
            .folder_repository
            .find(FolderFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        let items = build_netscape_hierarchy(&folders, &bookmarks, None);

        let netscape = Netscape {
            items,
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        self.netscape_manager
            .export(&mut raw, netscape)
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        Ok(raw.into())
    }
}

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + 'static {
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error>;

    async fn import_netscape(&self, outlines: Vec<Item>, user_id: Uuid) -> Result<(), Error>;
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
            title: feed
                .title
                .clone()
                .or_else(|| Some(feed.original_title.clone())),
            text: feed.original_title.clone(),
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

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Opml(#[from] OpmlError),

    #[error(transparent)]
    Netscape(#[from] NetscapeError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpmlError(#[from] anyhow::Error);

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct NetscapeError(#[from] anyhow::Error);
