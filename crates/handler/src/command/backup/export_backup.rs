use bytes::Bytes;
use colette_common::RepositoryError;
use colette_core::backup::Backup;
use uuid::Uuid;

use crate::{
    BookmarkQueryParams, BookmarkQueryRepository, Handler, SubscriptionQueryParams,
    SubscriptionQueryRepository, TagQueryParams, TagQueryRepository,
};

#[derive(Debug, Clone)]
pub struct ExportBackupCommand {
    pub user_id: Uuid,
}

pub struct ExportBackupHandler<
    BQR: BookmarkQueryRepository,
    SQR: SubscriptionQueryRepository,
    TQR: TagQueryRepository,
> {
    bookmark_query_repository: BQR,
    subscription_query_repository: SQR,
    tag_query_repository: TQR,
}

impl<BQR: BookmarkQueryRepository, SQR: SubscriptionQueryRepository, TQR: TagQueryRepository>
    ExportBackupHandler<BQR, SQR, TQR>
{
    pub fn new(
        bookmark_query_repository: BQR,
        subscription_query_repository: SQR,
        tag_query_repository: TQR,
    ) -> Self {
        Self {
            bookmark_query_repository,
            subscription_query_repository,
            tag_query_repository,
        }
    }
}

impl<BQR: BookmarkQueryRepository, SQR: SubscriptionQueryRepository, TQR: TagQueryRepository>
    Handler<ExportBackupCommand> for ExportBackupHandler<BQR, SQR, TQR>
{
    type Response = Bytes;
    type Error = ExportBackupError;

    async fn handle(&self, cmd: ExportBackupCommand) -> Result<Self::Response, Self::Error> {
        let subscriptions = self
            .subscription_query_repository
            .query(SubscriptionQueryParams {
                user_id: cmd.user_id,
                ..Default::default()
            })
            .await?;

        let bookmarks = self
            .bookmark_query_repository
            .query(BookmarkQueryParams {
                user_id: cmd.user_id,
                ..Default::default()
            })
            .await?;

        let tags = self
            .tag_query_repository
            .query(TagQueryParams {
                user_id: cmd.user_id,
                ..Default::default()
            })
            .await?;

        let backup = Backup {
            subscriptions: Vec::new(),
            bookmarks: Vec::new(),
            tags: Vec::new(), // subscriptions: subscriptions.into_iter().map(Into::into).collect(),
                              // bookmarks: bookmarks.into_iter().map(Into::into).collect(),
                              // tags: tags.into_iter().map(Into::into).collect(),
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
