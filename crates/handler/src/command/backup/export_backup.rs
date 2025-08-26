use bytes::Bytes;
use colette_archival::{Backup, BackupBookmark, BackupSubscription, BackupTag};
use colette_common::RepositoryError;
use uuid::Uuid;

use crate::{
    BookmarkDto, BookmarkQueryParams, BookmarkQueryRepository, Handler, SubscriptionDto,
    SubscriptionQueryParams, SubscriptionQueryRepository, TagDto, TagQueryParams,
    TagQueryRepository,
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
            subscriptions: subscriptions.into_iter().map(Into::into).collect(),
            bookmarks: bookmarks.into_iter().map(Into::into).collect(),
            tags: tags.into_iter().map(Into::into).collect(),
        };

        let raw = serde_json::to_vec_pretty(&backup)?;

        Ok(raw.into())
    }
}

impl From<SubscriptionDto> for BackupSubscription {
    fn from(value: SubscriptionDto) -> Self {
        Self {
            id: value.id,
            source_url: value.source_url,
            title: value.title,
            description: value.description,
            tags: value.tags.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<BookmarkDto> for BackupBookmark {
    fn from(value: BookmarkDto) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            archived_path: value.archived_path,
            tags: value.tags.into_iter().map(Into::into).collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

impl From<TagDto> for BackupTag {
    fn from(value: TagDto) -> Self {
        Self {
            id: value.id,
            title: value.title,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExportBackupError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
