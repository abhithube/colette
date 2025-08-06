use std::sync::Arc;

use bytes::Bytes;
use uuid::Uuid;

use super::{Backup, BackupRepository, Error};
use crate::{
    backup::ImportBackupParams,
    bookmark::{BookmarkFindParams, BookmarkRepository},
    subscription::{SubscriptionFindParams, SubscriptionRepository},
    tag::{TagFindParams, TagRepository},
};

pub struct BackupService {
    backup_repository: Arc<dyn BackupRepository>,
    bookmark_repository: Arc<dyn BookmarkRepository>,
    subscription_repository: Arc<dyn SubscriptionRepository>,
    tag_repository: Arc<dyn TagRepository>,
}

impl BackupService {
    pub fn new(
        backup_repository: Arc<dyn BackupRepository>,
        bookmark_repository: Arc<dyn BookmarkRepository>,
        subscription_repository: Arc<dyn SubscriptionRepository>,
        tag_repository: Arc<dyn TagRepository>,
    ) -> Self {
        Self {
            backup_repository,
            bookmark_repository,
            subscription_repository,
            tag_repository,
        }
    }

    pub async fn import_backup(&self, raw: Bytes, user_id: Uuid) -> Result<(), Error> {
        let backup = serde_json::from_slice::<Backup>(&raw)?;

        self.backup_repository
            .import(ImportBackupParams { backup, user_id })
            .await?;

        Ok(())
    }

    pub async fn export_backup(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                with_tags: true,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                with_tags: true,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let backup = Backup {
            subscriptions: subscriptions.into_iter().map(Into::into).collect(),
            bookmarks: bookmarks.into_iter().map(Into::into).collect(),
            tags: tags.into_iter().map(Into::into).collect(),
        };

        let raw = serde_json::to_vec_pretty(&backup)?;

        Ok(raw.into())
    }
}
