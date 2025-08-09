use bytes::Bytes;
use uuid::Uuid;

use super::Backup;
use crate::{
    Handler, RepositoryError,
    bookmark::{BookmarkFindParams, BookmarkRepository},
    subscription::{SubscriptionFindParams, SubscriptionRepository},
    tag::{TagFindParams, TagRepository},
};

#[derive(Debug, Clone)]
pub struct ExportBackupCommand {
    pub user_id: Uuid,
}

pub struct ExportBackupHandler {
    bookmark_repository: Box<dyn BookmarkRepository>,
    subscription_repository: Box<dyn SubscriptionRepository>,
    tag_repository: Box<dyn TagRepository>,
}

impl ExportBackupHandler {
    pub fn new(
        bookmark_repository: impl BookmarkRepository,
        subscription_repository: impl SubscriptionRepository,
        tag_repository: impl TagRepository,
    ) -> Self {
        Self {
            bookmark_repository: Box::new(bookmark_repository),
            subscription_repository: Box::new(subscription_repository),
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<ExportBackupCommand> for ExportBackupHandler {
    type Response = Bytes;
    type Error = ExportBackupError;

    async fn handle(&self, cmd: ExportBackupCommand) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                with_tags: true,
                user_id: Some(cmd.user_id),
                ..Default::default()
            })
            .await?;

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                with_tags: true,
                user_id: Some(cmd.user_id),
                ..Default::default()
            })
            .await?;

        let tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: Some(cmd.user_id),
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

#[derive(Debug, thiserror::Error)]
pub enum ExportBackupError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
