use std::sync::Arc;

use bytes::Bytes;
use uuid::Uuid;

use super::{Backup, BackupRepository, Error};
use crate::{
    backup::ImportBackupData,
    bookmark::{BookmarkParams, BookmarkRepository},
    subscription::{SubscriptionParams, SubscriptionRepository},
    tag::{TagParams, TagRepository},
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
            .import(ImportBackupData { backup, user_id })
            .await?;

        Ok(())
    }

    pub async fn export_backup(&self, user_id: Uuid) -> Result<Bytes, Error> {
        let subscriptions = self
            .subscription_repository
            .query(SubscriptionParams {
                with_feed: true,
                with_tags: true,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let bookmarks = self
            .bookmark_repository
            .query(BookmarkParams {
                with_tags: true,
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let tags = self
            .tag_repository
            .query(TagParams {
                user_id: Some(user_id),
                ..Default::default()
            })
            .await?;

        let backup = Backup {
            subscriptions,
            bookmarks,
            tags,
        };

        let raw = serde_json::to_vec_pretty(&backup)?;

        Ok(raw.into())
    }
}
