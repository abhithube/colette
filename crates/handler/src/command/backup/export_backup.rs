use bytes::Bytes;
use colette_core::{
    auth::UserId,
    backup::Backup,
    bookmark::{BookmarkFindParams, BookmarkRepository},
    common::RepositoryError,
    subscription::{SubscriptionFindParams, SubscriptionRepository},
    tag::{TagFindParams, TagRepository},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct ExportBackupCommand {
    pub user_id: UserId,
}

pub struct ExportBackupHandler<
    BR: BookmarkRepository,
    SR: SubscriptionRepository,
    TR: TagRepository,
> {
    bookmark_repository: BR,
    subscription_repository: SR,
    tag_repository: TR,
}

impl<BR: BookmarkRepository, SR: SubscriptionRepository, TR: TagRepository>
    ExportBackupHandler<BR, SR, TR>
{
    pub fn new(bookmark_repository: BR, subscription_repository: SR, tag_repository: TR) -> Self {
        Self {
            bookmark_repository,
            subscription_repository,
            tag_repository,
        }
    }
}

#[async_trait::async_trait]
impl<BR: BookmarkRepository, SR: SubscriptionRepository, TR: TagRepository>
    Handler<ExportBackupCommand> for ExportBackupHandler<BR, SR, TR>
{
    type Response = Bytes;
    type Error = ExportBackupError;

    async fn handle(&self, cmd: ExportBackupCommand) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .subscription_repository
            .find(SubscriptionFindParams {
                user_id: cmd.user_id,
                id: None,
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;

        let bookmarks = self
            .bookmark_repository
            .find(BookmarkFindParams {
                user_id: cmd.user_id,
                id: None,
                filter: None,
                tags: None,
                cursor: None,
                limit: None,
            })
            .await?;

        let tags = self
            .tag_repository
            .find(TagFindParams {
                user_id: cmd.user_id,
                id: None,
                cursor: None,
                limit: None,
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
