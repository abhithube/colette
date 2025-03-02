use std::{collections::HashMap, sync::Arc};

use bytes::{Buf, Bytes};
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use tokio::sync::Mutex;
use url::Url;
use uuid::Uuid;

use super::{Error, backup_repository::BackupRepository};
use crate::{
    bookmark::{BookmarkFindParams, BookmarkRepository},
    feed::{FeedFindParams, FeedRepository},
    job::Storage,
};

pub struct BackupService {
    backup_repository: Box<dyn BackupRepository>,
    feed_repository: Box<dyn FeedRepository>,
    bookmark_repository: Box<dyn BookmarkRepository>,
    import_feeds_storage: Arc<Mutex<dyn Storage<ImportFeedsJob>>>,
    import_bookmarks_storage: Arc<Mutex<dyn Storage<ImportBookmarksJob>>>,
}

impl BackupService {
    pub fn new(
        backup_repository: impl BackupRepository,
        feed_repository: impl FeedRepository,
        bookmark_repository: impl BookmarkRepository,
        import_feeds_storage: Arc<Mutex<dyn Storage<ImportFeedsJob>>>,
        import_bookmarks_storage: Arc<Mutex<dyn Storage<ImportBookmarksJob>>>,
    ) -> Self {
        Self {
            backup_repository: Box::new(backup_repository),
            feed_repository: Box::new(feed_repository),
            bookmark_repository: Box::new(bookmark_repository),
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

        self.backup_repository
            .import_feeds(opml.body.outlines, user_id)
            .await?;

        let mut storage = self.import_feeds_storage.lock().await;
        storage.push(ImportFeedsJob { urls }).await?;

        Ok(())
    }

    pub async fn export_opml(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let mut outline_map = HashMap::<Uuid, Outline>::new();

        let feeds = self
            .feed_repository
            .find_feeds(FeedFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .unwrap();

        for feed in feeds {
            let outline = Outline {
                r#type: Some(OutlineType::default()),
                text: feed.title.clone(),
                xml_url: feed.xml_url.map(Into::into),
                title: Some(feed.title),
                html_url: Some(feed.link.into()),
                ..Default::default()
            };

            if let Some(tags) = feed.tags {
                for tag in tags {
                    outline_map
                        .entry(tag.id)
                        .or_insert_with(|| Outline {
                            text: tag.title,
                            ..Default::default()
                        })
                        .outline
                        .push(outline.clone());
                }
            }
        }

        let opml = Opml {
            body: Body {
                outlines: outline_map.into_values().collect(),
            },
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

        self.backup_repository
            .import_bookmarks(netscape.items, user_id)
            .await?;

        let mut storage = self.import_bookmarks_storage.lock().await;
        storage.push(ImportBookmarksJob { urls, user_id }).await?;

        Ok(())
    }

    pub async fn export_netscape(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let mut item_map = HashMap::<Uuid, Item>::new();

        let bookmarks = self
            .bookmark_repository
            .find_bookmarks(BookmarkFindParams {
                user_id,
                ..Default::default()
            })
            .await
            .unwrap();

        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                add_date: bookmark.created_at.map(|e| e.timestamp()),
                last_modified: bookmark.updated_at.map(|e| e.timestamp()),
                href: Some(bookmark.link.into()),
                ..Default::default()
            };

            if let Some(tags) = bookmark.tags {
                for tag in tags {
                    item_map
                        .entry(tag.id)
                        .or_insert_with(|| Item {
                            title: tag.title,
                            ..Default::default()
                        })
                        .item
                        .push(item.clone());
                }
            }
        }

        let netscape = Netscape {
            items: item_map.into_values().collect(),
            ..Default::default()
        };

        let mut raw = Vec::<u8>::new();

        colette_netscape::to_writer(&mut raw, netscape)?;

        Ok(raw.into())
    }
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
