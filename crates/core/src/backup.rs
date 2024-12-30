use std::collections::HashMap;

use bytes::Bytes;
use colette_backup::BackupManager;
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use dyn_clone::DynClone;
use url::Url;
use uuid::Uuid;

use crate::{
    bookmark::{BookmarkFindParams, BookmarkRepository},
    feed::{FeedFindParams, FeedRepository},
};

#[derive(Clone)]
pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    feed_repository: Box<dyn FeedRepository>,
    bookmark_repository: Box<dyn BookmarkRepository>,
    opml_manager: Box<dyn BackupManager<Data = Opml>>,
    netscape_manager: Box<dyn BackupManager<Data = Netscape>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        feed_repository: impl FeedRepository,
        bookmark_repository: impl BookmarkRepository,
        opml_manager: impl BackupManager<Data = Opml>,
        netscape_manager: impl BackupManager<Data = Netscape>,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            feed_repository: Box::new(feed_repository),
            bookmark_repository: Box::new(bookmark_repository),
            opml_manager: Box::new(opml_manager),
            netscape_manager: Box::new(netscape_manager),
        }
    }

    pub async fn import_opml(&self, raw: Bytes, user_id: Uuid) -> Result<Vec<Url>, Error> {
        let opml = self
            .opml_manager
            .import(raw)
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
        let feeds = self
            .feed_repository
            .find(FeedFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let mut tag_map: HashMap<Uuid, Outline> = HashMap::new();
        let mut root_feeds: Vec<Outline> = Vec::new();

        for feed in feeds {
            let outline = Outline {
                r#type: Some(OutlineType::default()),
                title: feed.title.or_else(|| Some(feed.original_title.clone())),
                text: feed.original_title,
                xml_url: feed.url,
                html_url: Some(feed.link),
                ..Default::default()
            };

            if let Some(tags) = feed.tags {
                for tag in tags {
                    let root_tag = tag_map.entry(tag.id).or_insert_with(|| Outline {
                        text: tag.title,
                        ..Default::default()
                    });

                    root_tag
                        .outline
                        .get_or_insert_with(Vec::new)
                        .push(outline.clone());
                }
            } else {
                root_feeds.push(outline);
            }
        }

        let mut outlines = tag_map.into_values().collect::<Vec<_>>();
        outlines.sort();
        outlines.append(&mut root_feeds);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        self.opml_manager
            .export(opml)
            .map_err(|e| Error::Opml(OpmlError(e.into())))
    }

    pub async fn import_netscape(&self, raw: Bytes, user_id: Uuid) -> Result<Vec<Url>, Error> {
        let netscape = self
            .netscape_manager
            .import(raw)
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
        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        let mut tag_map: HashMap<Uuid, Item> = HashMap::new();
        let mut root_bookmarks: Vec<Item> = Vec::new();

        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                href: Some(bookmark.link),
                ..Default::default()
            };

            if let Some(tags) = bookmark.tags {
                for tag in tags {
                    let root_tag = tag_map.entry(tag.id).or_insert_with(|| Item {
                        title: tag.title,
                        ..Default::default()
                    });

                    root_tag
                        .item
                        .get_or_insert_with(Vec::new)
                        .push(item.clone());
                }
            } else {
                root_bookmarks.push(item);
            }
        }

        let mut items = tag_map.into_values().collect::<Vec<_>>();
        items.sort();
        items.append(&mut root_bookmarks);

        let netscape = Netscape {
            items,
            ..Default::default()
        };

        self.netscape_manager
            .export(netscape)
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))
    }
}

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync + DynClone + 'static {
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error>;

    async fn import_netscape(&self, outlines: Vec<Item>, user_id: Uuid) -> Result<(), Error>;
}

dyn_clone::clone_trait_object!(BackupRepository);

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
