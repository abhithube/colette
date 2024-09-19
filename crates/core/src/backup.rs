use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use bytes::Bytes;
use colette_backup::BackupManager;
use colette_netscape::{Item, Netscape};
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{Bookmark, Collection, Feed, Tag};

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    opml_manager: Arc<dyn BackupManager<T = Opml>>,
    netscape_manager: Arc<dyn BackupManager<T = Netscape>>,
}

#[derive(Clone, Debug)]
pub struct ItemWrapper {
    pub parent_id: Option<Uuid>,
    pub item: Item,
}

impl BackupService {
    pub fn new(
        backup_repository: Arc<dyn BackupRepository>,
        opml_manager: Arc<dyn BackupManager<T = Opml>>,
        netscape_manager: Arc<dyn BackupManager<T = Netscape>>,
    ) -> Self {
        Self {
            backup_repository,
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
        let (tags, feeds) = self.backup_repository.export_opml(profile_id).await?;

        let mut tag_map: BTreeMap<String, Outline> = BTreeMap::new();

        for tag in tags {
            tag_map.insert(
                tag.title.clone(),
                Outline {
                    text: tag.title,
                    ..Default::default()
                },
            );
        }

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
                    if let Some(parent) = tag_map.get_mut(&tag.title) {
                        parent
                            .outline
                            .get_or_insert_with(Vec::new)
                            .push(outline.clone());
                    }
                }
            } else {
                root_feeds.push(outline);
            }
        }

        let mut outlines = tag_map.into_values().collect::<Vec<_>>();
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
        let (collections, bookmarks) = self.backup_repository.export_netscape(profile_id).await?;

        let mut collection_map: HashMap<Uuid, ItemWrapper> = HashMap::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut root_collections: Vec<Uuid> = vec![];

        for collection in collections {
            collection_map.insert(
                collection.id,
                ItemWrapper {
                    parent_id: collection.parent_id,
                    item: Item {
                        title: collection.title,
                        ..Default::default()
                    },
                },
            );

            match collection.parent_id {
                Some(parent_id) => {
                    children_map
                        .entry(parent_id)
                        .or_default()
                        .push(collection.id);
                }
                None => root_collections.push(collection.id),
            }
        }

        let mut root_bookmarks: Vec<Item> = vec![];
        for bookmark in bookmarks {
            let item = Item {
                title: bookmark.title,
                href: Some(bookmark.link),
                tags: bookmark
                    .tags
                    .map(|e| e.into_iter().map(|e| e.title).collect()),
                ..Default::default()
            };

            match bookmark.collection_id {
                Some(collection_id) => {
                    if let Some(parent) = collection_map.get_mut(&collection_id) {
                        parent.item.item.get_or_insert_with(Vec::new).push(item);
                    }
                }
                None => root_bookmarks.push(item),
            }
        }

        fn build_hierarchy(
            folder_map: &mut HashMap<Uuid, ItemWrapper>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
            folder_id: Uuid,
        ) {
            if let Some(children) = children_map.get(&folder_id) {
                for &child_id in children {
                    build_hierarchy(folder_map, children_map, child_id);
                    if let Some(child) = folder_map.remove(&child_id) {
                        if let Some(children) =
                            folder_map.get_mut(&folder_id).unwrap().item.item.as_mut()
                        {
                            children.push(child.item);
                        }
                    }
                }
            }
        }

        for &root_id in &root_collections {
            build_hierarchy(&mut collection_map, &children_map, root_id);
        }

        let mut items = root_collections
            .into_iter()
            .filter_map(|id| collection_map.remove(&id).map(|e| e.item))
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

    async fn export_opml(&self, profile_id: Uuid) -> Result<(Vec<Tag>, Vec<Feed>), Error>;

    async fn import_netscape(&self, outlines: Vec<Item>, profile_id: Uuid) -> Result<(), Error>;

    async fn export_netscape(
        &self,
        profile_id: Uuid,
    ) -> Result<(Vec<Collection>, Vec<Bookmark>), Error>;
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
