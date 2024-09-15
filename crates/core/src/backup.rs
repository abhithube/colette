use std::{
    collections::HashMap,
    io::{Read, Write},
    sync::Arc,
};

use colette_backup::BackupManager;
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{
    feed::FeedRepository,
    folder::{FolderFindManyFilters, FolderRepository, FolderType},
};

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    feed_repository: Arc<dyn FeedRepository>,
    folder_repository: Arc<dyn FolderRepository>,
    opml_manager: Arc<dyn BackupManager<T = Opml>>,
}

#[derive(Clone, Debug)]
pub struct OutlineWrapper {
    pub parent_id: Option<Uuid>,
    pub outline: Outline,
}

impl BackupService {
    pub fn new(
        backup_repository: Arc<dyn BackupRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        folder_repository: Arc<dyn FolderRepository>,
        opml_manager: Arc<dyn BackupManager<T = Opml>>,
    ) -> Self {
        Self {
            backup_repository,
            feed_repository,
            folder_repository,
            opml_manager,
        }
    }

    pub async fn import_opml<R: Read + 'static>(
        &self,
        reader: R,
        profile_id: Uuid,
    ) -> Result<(), Error> {
        let opml = self
            .opml_manager
            .import(Box::new(reader))
            .map_err(|e| Error::Opml(OpmlError(e.into())))?;

        self.backup_repository
            .import_opml(opml.body.outlines, profile_id)
            .await
    }

    pub async fn export_opml<W: Write>(
        &self,
        mut writer: W,
        profile_id: Uuid,
    ) -> Result<(), Error> {
        let folders = self
            .folder_repository
            .list(
                profile_id,
                None,
                None,
                Some(FolderFindManyFilters {
                    folder_type: FolderType::Feeds,
                }),
            )
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        let feeds = self
            .feed_repository
            .list(profile_id, None, None, None)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut folder_map: HashMap<Uuid, OutlineWrapper> = HashMap::new();
        let mut children_map: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        let mut root_folders: Vec<Uuid> = vec![];

        for folder in folders {
            folder_map.insert(
                folder.id,
                OutlineWrapper {
                    parent_id: folder.parent_id,
                    outline: Outline {
                        text: folder.title,
                        ..Default::default()
                    },
                },
            );

            match folder.parent_id {
                Some(parent_id) => {
                    children_map.entry(parent_id).or_default().push(folder.id);
                }
                None => root_folders.push(folder.id),
            }
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

            match feed.folder_id {
                Some(folder_id) => {
                    if let Some(parent) = folder_map.get_mut(&folder_id) {
                        if let Some(children) = parent.outline.outline.as_mut() {
                            children.push(outline);
                        }
                    }
                }
                None => root_feeds.push(outline),
            }
        }

        fn build_hierarchy(
            folder_map: &mut HashMap<Uuid, OutlineWrapper>,
            children_map: &HashMap<Uuid, Vec<Uuid>>,
            folder_id: Uuid,
        ) {
            if let Some(children) = children_map.get(&folder_id) {
                for &child_id in children {
                    build_hierarchy(folder_map, children_map, child_id);
                    if let Some(child) = folder_map.remove(&child_id) {
                        if let Some(children) = folder_map
                            .get_mut(&folder_id)
                            .unwrap()
                            .outline
                            .outline
                            .as_mut()
                        {
                            children.push(child.outline);
                        }
                    }
                }
            }
        }

        for &root_id in &root_folders {
            build_hierarchy(&mut folder_map, &children_map, root_id);
        }

        let mut outlines = root_folders
            .into_iter()
            .filter_map(|id| folder_map.remove(&id).map(|e| e.outline))
            .collect::<Vec<_>>();
        outlines.append(&mut root_feeds);

        let opml = Opml {
            body: Body { outlines },
            ..Default::default()
        };

        self.opml_manager
            .export(&mut writer, opml)
            .map_err(|e| Error::Opml(OpmlError(e.into())))
    }
}

#[async_trait::async_trait]
pub trait BackupRepository: Send + Sync {
    async fn import_opml(&self, outlines: Vec<Outline>, profile_id: Uuid) -> Result<(), Error>;
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Opml(#[from] OpmlError),

    #[error(transparent)]
    Unknown(#[from] anyhow::Error),
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct OpmlError(#[from] anyhow::Error);
