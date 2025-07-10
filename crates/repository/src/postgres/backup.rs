use std::collections::HashMap;

use chrono::Utc;
use colette_core::{
    Feed, Tag,
    backup::{BackupRepository, Error, ImportBackupData},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    bookmark::{BookmarkBase, BookmarkDelete, BookmarkInsert},
    bookmark_tag::{BookmarkTagBase, BookmarkTagInsert},
    feed::{FeedBase, FeedInsert, FeedSelect},
    subscription::{SubscriptionBase, SubscriptionDelete, SubscriptionInsert},
    subscription_tag::{SubscriptionTagBase, SubscriptionTagInsert},
    tag::{TagBase, TagDelete, TagInsert, TagSelect},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder as _;
use url::Url;
use uuid::Uuid;

use super::PreparedClient as _;

#[derive(Debug, Clone)]
pub struct PostgresBackupRepository {
    pool: Pool,
}

impl PostgresBackupRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BackupRepository for PostgresBackupRepository {
    async fn import(&self, data: ImportBackupData) -> Result<(), Error> {
        let mut client = self.pool.get().await?;
        let tx = client.transaction().await?;

        let tag_map = {
            let (sql, values) = TagDelete {
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let tags = data
                .backup
                .tags
                .iter()
                .map(|e| TagBase {
                    id: Uuid::new_v4(),
                    title: &e.title,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                })
                .collect::<Vec<_>>();

            let (sql, values) = TagInsert {
                tags,
                user_id: data.user_id,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let (sql, values) = TagSelect {
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_select()
            .build_postgres(PostgresQueryBuilder);
            let tags = tx.query_prepared::<Tag>(&sql, &values).await?;

            tags.into_iter()
                .map(|e| (e.title, e.id))
                .collect::<HashMap<_, _>>()
        };

        {
            let (sql, values) = SubscriptionDelete {
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let feed_map = {
                let mut source_urls = Vec::<Url>::new();
                let mut feeds = Vec::<FeedBase>::new();

                for subscription in data.backup.subscriptions.iter() {
                    if let Some(ref feed) = subscription.feed {
                        feeds.push(FeedBase {
                            id: Uuid::new_v4(),
                            source_url: feed.source_url.as_str(),
                            link: feed.link.as_str(),
                            title: &feed.title,
                            description: feed.description.as_deref(),
                            refreshed_at: feed.refreshed_at,
                            is_custom: feed.is_custom,
                        });

                        source_urls.push(feed.source_url.clone());
                    }
                }

                let (sql, values) = FeedInsert {
                    feeds,
                    upsert: false,
                }
                .into_insert()
                .build_postgres(PostgresQueryBuilder);
                tx.execute_prepared(&sql, &values).await?;

                let (sql, values) = FeedSelect {
                    source_urls: Some(source_urls.iter().map(|e| e.as_str()).collect()),
                    ..Default::default()
                }
                .into_select()
                .build_postgres(PostgresQueryBuilder);
                let feeds = tx.query_prepared::<Feed>(&sql, &values).await?;

                feeds
                    .into_iter()
                    .map(|e| (e.source_url.clone(), e))
                    .collect::<HashMap<_, _>>()
            };

            let mut subscriptions = Vec::<SubscriptionBase>::new();
            let mut subscription_tags = Vec::<SubscriptionTagBase<Vec<Uuid>>>::new();

            for subscription in data.backup.subscriptions.iter() {
                if let Some(ref f) = subscription.feed
                    && let Some(feed) = feed_map.get(&f.source_url)
                {
                    let id = Uuid::new_v4();

                    subscriptions.push(SubscriptionBase {
                        id,
                        title: &subscription.title,
                        description: subscription.description.as_deref(),
                        feed_id: feed.id,
                        created_at: subscription.created_at,
                        updated_at: subscription.updated_at,
                    });

                    if let Some(tag) = subscription.tags.as_deref() {
                        let tag_ids = tag
                            .iter()
                            .flat_map(|e| tag_map.get(&e.title).copied())
                            .collect::<Vec<_>>();

                        subscription_tags.push(SubscriptionTagBase {
                            subscription_id: id,
                            user_id: data.user_id,
                            tag_ids,
                        });
                    }
                }
            }

            let (sql, values) = SubscriptionInsert {
                subscriptions,
                user_id: data.user_id,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let (sql, values) = SubscriptionTagInsert { subscription_tags }
                .into_insert()
                .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;
        }

        {
            let (sql, values) = BookmarkDelete {
                user_id: Some(data.user_id),
                ..Default::default()
            }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            let mut bookmarks = Vec::<BookmarkBase>::new();
            let mut bookmark_tags = Vec::<BookmarkTagBase<Vec<Uuid>>>::new();

            for bookmark in data.backup.bookmarks.iter() {
                let id = Uuid::new_v4();

                bookmarks.push(BookmarkBase {
                    id,
                    link: bookmark.link.as_str(),
                    title: &bookmark.title,
                    thumbnail_url: bookmark.thumbnail_url.as_ref().map(|e| e.as_str()),
                    published_at: bookmark.published_at,
                    author: bookmark.author.as_deref(),
                    archived_path: bookmark.archived_path.as_deref(),
                    created_at: bookmark.created_at,
                    updated_at: bookmark.updated_at,
                });

                if let Some(tag) = bookmark.tags.as_deref() {
                    let tag_ids = tag
                        .iter()
                        .flat_map(|e| tag_map.get(&e.title).copied())
                        .collect::<Vec<_>>();

                    bookmark_tags.push(BookmarkTagBase {
                        bookmark_id: id,
                        tag_ids,
                    });
                }
            }

            BookmarkInsert {
                bookmarks,
                user_id: data.user_id,
                upsert: false,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;

            BookmarkTagInsert {
                bookmark_tags,
                user_id: data.user_id,
            }
            .into_insert()
            .build_postgres(PostgresQueryBuilder);
            tx.execute_prepared(&sql, &values).await?;
        }

        tx.commit().await?;

        Ok(())
    }
}
