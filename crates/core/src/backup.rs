use std::sync::Arc;

use bytes::Bytes;
use colette_backup::BackupManager;
use opml::{Body, Outline, OPML};
use uuid::Uuid;

use crate::feed::FeedRepository;

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    feed_repository: Arc<dyn FeedRepository>,
    opml_manager: Arc<dyn BackupManager<T = OPML>>,
}

impl BackupService {
    pub fn new(
        backup_repository: Arc<dyn BackupRepository>,
        feed_repository: Arc<dyn FeedRepository>,
        opml_manager: Arc<dyn BackupManager<T = OPML>>,
    ) -> Self {
        Self {
            backup_repository,
            feed_repository,
            opml_manager,
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
            .map_err(|e| Error::Unknown(e.into()))?;

        let data = feeds
            .data
            .iter()
            .cloned()
            .map(|e| Outline {
                r#type: Some("rss".to_owned()),
                text: e.original_title,
                title: e.title,
                xml_url: e.url,
                html_url: Some(e.link),
                ..Default::default()
            })
            .collect::<Vec<_>>();

        let opml = OPML {
            version: "2.0".to_owned(),
            body: Body { outlines: data },
            ..Default::default()
        };

        self.opml_manager
            .export(opml)
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
