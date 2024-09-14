use std::{fmt::Write, io::Read, sync::Arc};

use colette_backup::BackupManager;
use colette_opml::{Body, Opml, Outline, OutlineType};
use uuid::Uuid;

use crate::{feed::FeedRepository, folder::FolderRepository};

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    feed_repository: Arc<dyn FeedRepository>,
    folder_repository: Arc<dyn FolderRepository>,
    opml_manager: Arc<dyn BackupManager<T = Opml>>,
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

    pub async fn export_opml<W: Write + Send>(
        &self,
        mut writer: W,
        profile_id: Uuid,
    ) -> Result<(), Error> {
        let feeds = self
            .feed_repository
            .list(profile_id, None, None, None)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = feeds
            .iter()
            .cloned()
            .map(|e| Outline {
                r#type: Some(OutlineType::default()),
                title: e.title.or_else(|| Some(e.original_title.clone())),
                text: e.original_title,
                xml_url: e.url,
                html_url: Some(e.link),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let opml = Opml {
            body: Body { outlines: data },
            ..Default::default()
        };

        self.opml_manager
            .export(opml, &mut writer)
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
