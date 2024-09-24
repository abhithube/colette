use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use bytes::Bytes;
use colette_backup::BackupManager;
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{bookmark::BookmarkRepository, feed::FeedRepository};

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    feed_repository: Arc<dyn FeedRepository>,
    bookmark_repository: Arc<dyn BookmarkRepository>,
    opml_manager: Arc<dyn BackupManager<T = Opml>>,
    netscape_manager: Arc<dyn BackupManager<T = Netscape>>,
}

#[derive(Clone, Debug)]
pub struct OutlineWrapper {
    pub parent_id: Option<Uuid>,
    pub outline: Outline,
}

#[derive(Clone, Debug)]
pub struct ItemWrapper {
    pub parent_id: Option<Uuid>,
    pub item: Item,
}

impl BackupService {
    pub fn new(
        backup_repository: Arc<dyn BackupRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        bookmark_repository: Arc<dyn BookmarkRepository>,
        opml_manager: Arc<dyn BackupManager<T = Opml>>,
        netscape_manager: Arc<dyn BackupManager<T = Netscape>>,
    ) -> Self {
        Self {
            backup_repository,
            feed_repository,
            bookmark_repository,
            opml_manager,
            netscape_manager,
        }
    }

    pub async fn import_opml(&self, raw: Bytes, profile_id: Uuid) -> Result<(), Error> {
        let opml = self
            .opml_manager
            .import(raw)
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        self.backup_repository
            .import_opml(opml.body.outlines, profile_id)
            .await
    }

    pub async fn export_opml(&self, profile_id: Uuid) -> Result<Bytes, Error> {
        let feeds = self
            .feed_repository
            .list(profile_id, None, None, None)
            .await
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        let mut tag_map: HashMap<Uuid, OutlineWrapper> = HashMap::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut root_tags: HashSet<Uuid> = HashSet::new();

        let mut root_feeds: Vec<Outline> = vec![];
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
                    let parent = tag_map.entry(tag.id).or_insert_with(|| OutlineWrapper {
                        parent_id: tag.parent_id,
                        outline: Outline {
                            text: tag.title,
                            ..Default::default()
                        },
                    });

                    match tag.parent_id {
                        Some(parent_id) => {
                            children_map.entry(parent_id).or_default().push(tag.id);
                        }
                        None => {
                            root_tags.insert(tag.id);
                        }
                    }

                    if tag.direct.is_some_and(|e| e) {
                        parent
                            .outline
                            .outline
                            .get_or_insert_with(Vec::new)
                            .push(outline.clone());
                    }
                }
            } else {
                root_feeds.push(outline);
            }
        }

        fn build_hierarchy(
            tag_map: &mut HashMap<Uuid, OutlineWrapper>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
            root_id: &Uuid,
        ) {
            if let Some(children) = children_map.get(root_id) {
                for child_id in children {
                    if let Some(child) = tag_map.remove(child_id) {
                        if let Some(wrapper) = tag_map.get_mut(root_id) {
                            let outlines = wrapper.outline.outline.get_or_insert_with(Vec::new);

                            outlines.push(child.outline);
                            outlines.sort();
                        }
                    }

                    build_hierarchy(tag_map, children_map, child_id);
                }
            }
        }

        for root_id in &root_tags {
            build_hierarchy(&mut tag_map, &children_map, root_id);
        }

        let mut outlines = root_tags
            .into_iter()
            .filter_map(|id| tag_map.remove(&id).map(|e| e.outline))
            .collect::<Vec<_>>();
        outlines.append(&mut root_feeds);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        self.opml_manager
            .export(opml)
            .map_err(|e| Error::Opml(OpmlError(e.into())))
    }

    pub async fn import_netscape(&self, raw: Bytes, profile_id: Uuid) -> Result<(), Error> {
        let netscape = self
            .netscape_manager
            .import(raw)
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        self.backup_repository
            .import_netscape(netscape.items, profile_id)
            .await
    }

    pub async fn export_netscape(&self, profile_id: Uuid) -> Result<Bytes, Error> {
        let bookmarks = self
            .bookmark_repository
            .list(profile_id, None, None, None)
            .await
            .map_err(|e| Error::Netscape(NetscapeError(e.into())))?;

        let mut tag_map: HashMap<Uuid, ItemWrapper> = HashMap::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut root_tags: HashSet<Uuid> = HashSet::new();

        let mut root_bookmarks: Vec<Item> = vec![];
        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                href: Some(bookmark.link),
                ..Default::default()
            };

            if let Some(tags) = bookmark.tags {
                for tag in tags {
                    let parent = tag_map.entry(tag.id).or_insert_with(|| ItemWrapper {
                        parent_id: tag.parent_id,
                        item: Item {
                            title: tag.title,
                            ..Default::default()
                        },
                    });

                    match tag.parent_id {
                        Some(parent_id) => {
                            children_map.entry(parent_id).or_default().push(tag.id);
                        }
                        None => {
                            root_tags.insert(tag.id);
                        }
                    }

                    if tag.direct.is_some_and(|e| e) {
                        parent
                            .item
                            .item
                            .get_or_insert_with(Vec::new)
                            .push(item.clone());
                    }
                }
            } else {
                root_bookmarks.push(item);
            }
        }

        fn build_hierarchy(
            tag_map: &mut HashMap<Uuid, ItemWrapper>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
            root_id: &Uuid,
        ) {
            if let Some(children) = children_map.get(root_id) {
                for child_id in children {
                    if let Some(child) = tag_map.remove(child_id) {
                        if let Some(wrapper) = tag_map.get_mut(root_id) {
                            let items = wrapper.item.item.get_or_insert_with(Vec::new);

                            items.push(child.item);
                            items.sort();
                        }
                    }

                    build_hierarchy(tag_map, children_map, child_id);
                }
            }
        }

        for root_id in &root_tags {
            build_hierarchy(&mut tag_map, &children_map, root_id);
        }

        let mut items = root_tags
            .into_iter()
            .filter_map(|id| tag_map.remove(&id).map(|e| e.item))
            .collect::<Vec<_>>();
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
pub trait BackupRepository: Send + Sync {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error>;

    async fn import_netscape(&self, outlines: Vec<Item>, profile_id: Uuid) -> Result<(), Error>;
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
