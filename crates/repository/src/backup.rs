use chrono::{DateTime, Utc};
use colette_core::{
    backup::{BackupRepository, ImportBackupParams},
    common::RepositoryError,
};
use sqlx::PgPool;

use crate::DbUrl;

#[derive(Debug, Clone)]
pub struct PostgresBackupRepository {
    pool: PgPool,
}

impl PostgresBackupRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BackupRepository for PostgresBackupRepository {
    async fn import(&self, params: ImportBackupParams) -> Result<(), RepositoryError> {
        let mut bookmark_links = Vec::<DbUrl>::new();
        let mut bookmark_titles = Vec::<String>::new();
        let mut bookmark_thumbnail_urls = Vec::<Option<DbUrl>>::new();
        let mut bookmark_published_ats = Vec::<Option<DateTime<Utc>>>::new();
        let mut bookmark_authors = Vec::<Option<String>>::new();
        let mut bookmark_archived_paths = Vec::<Option<String>>::new();
        let mut bookmark_created_ats = Vec::<DateTime<Utc>>::new();
        let mut bookmark_updated_ats = Vec::<DateTime<Utc>>::new();

        let mut bt_bookmark_links = Vec::<DbUrl>::new();
        let mut bt_tag_titles = Vec::<String>::new();

        for bookmark in params.backup.bookmarks {
            let link = DbUrl(bookmark.link);

            for tag in bookmark.tags {
                bt_bookmark_links.push(link.clone());
                bt_tag_titles.push(tag.title);
            }

            bookmark_links.push(link);
            bookmark_titles.push(bookmark.title);
            bookmark_thumbnail_urls.push(bookmark.thumbnail_url.map(Into::into));
            bookmark_published_ats.push(bookmark.published_at);
            bookmark_authors.push(bookmark.author);
            bookmark_archived_paths.push(bookmark.archived_path);
            bookmark_created_ats.push(bookmark.created_at);
            bookmark_updated_ats.push(bookmark.updated_at);
        }

        let mut feed_source_urls = Vec::<DbUrl>::new();
        let mut subscription_titles = Vec::<String>::new();
        let mut subscription_descriptions = Vec::<Option<String>>::new();
        let mut subscription_created_ats = Vec::<DateTime<Utc>>::new();
        let mut subscription_updated_ats = Vec::<DateTime<Utc>>::new();

        let mut st_feed_source_urls = Vec::<DbUrl>::new();
        let mut st_tag_titles = Vec::<String>::new();

        for subscription in params.backup.subscriptions {
            let source_url = DbUrl(subscription.source_url);

            for tag in subscription.tags {
                st_feed_source_urls.push(source_url.clone());
                st_tag_titles.push(tag.title);
            }

            feed_source_urls.push(source_url);
            subscription_titles.push(subscription.title);
            subscription_descriptions.push(subscription.description);
            subscription_created_ats.push(subscription.created_at);
            subscription_updated_ats.push(subscription.updated_at);
        }

        let mut tag_titles = Vec::<String>::new();
        let mut tag_created_ats = Vec::<DateTime<Utc>>::new();
        let mut tag_updated_ats = Vec::<DateTime<Utc>>::new();

        for tag in params.backup.tags {
            tag_titles.push(tag.title);
            tag_created_ats.push(tag.created_at);
            tag_updated_ats.push(tag.updated_at);
        }

        sqlx::query_file!(
            "queries/backups/import.sql",
            params.user_id.as_inner(),
            &feed_source_urls as &[DbUrl],
            &subscription_titles,
            &subscription_descriptions as &[Option<String>],
            &subscription_created_ats,
            &subscription_updated_ats,
            &bookmark_links as &[DbUrl],
            &bookmark_titles,
            &bookmark_thumbnail_urls as &[Option<DbUrl>],
            &bookmark_published_ats as &[Option<DateTime<Utc>>],
            &bookmark_authors as &[Option<String>],
            &bookmark_archived_paths as &[Option<String>],
            &bookmark_created_ats,
            &bookmark_updated_ats,
            &tag_titles,
            &tag_created_ats,
            &tag_updated_ats,
            &st_feed_source_urls as &[DbUrl],
            &st_tag_titles,
            &bt_bookmark_links as &[DbUrl],
            &bt_tag_titles,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
