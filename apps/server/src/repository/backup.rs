use colette_core::{
    backup::{BackupRepository, Error, ImportBookmarksParams, ImportFeedsParams},
    bookmark::{BookmarkUpsertParams, ProcessedBookmark},
};
use colette_query::{
    IntoInsert, IntoSelect, bookmark_tag::BookmarkTagUpsert, feed::FeedUpsert,
    subscription::SubscriptionUpsert, subscription_tag::SubscriptionTagUpsert, tag::TagUpsert,
};
use futures::TryFutureExt;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Row, Sqlite};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteBackupRepository {
    pool: Pool<Sqlite>,
}

impl SqliteBackupRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BackupRepository for SqliteBackupRepository {
    async fn import_feeds(&self, params: ImportFeedsParams) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = params
            .outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                let tag_id = {
                    let tag = TagUpsert {
                        id: Uuid::new_v4(),
                        title: outline.text,
                        user_id: params.user_id,
                    };

                    let (sql, values) = tag.clone().into_select().build_sqlx(SqliteQueryBuilder);

                    let row = sqlx::query_with(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await?;

                    match row {
                        Some(row) => row.get::<String, _>(0).parse().unwrap(),
                        _ => {
                            let id = tag.id;

                            let (sql, values) = tag.into_insert().build_sqlx(SqliteQueryBuilder);

                            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

                            id
                        }
                    }
                };

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let title = outline.title.unwrap_or(outline.text);

                let feed = FeedUpsert {
                    id: Uuid::new_v4(),
                    link: link.parse().unwrap(),
                    xml_url: outline.xml_url.map(|e| e.parse().unwrap()),
                    title: title.clone(),
                    description: None,
                    refreshed_at: None,
                };

                let (sql, values) = feed.into_insert().build_sqlx(SqliteQueryBuilder);

                let feed_id = sqlx::query_scalar_with::<_, String, _>(&sql, values)
                    .fetch_one(&mut *tx)
                    .map_ok(|e| e.parse().unwrap())
                    .await?;

                let subscription_id = {
                    let subscription = SubscriptionUpsert {
                        id: Uuid::new_v4(),
                        title,
                        feed_id,
                        user_id: params.user_id,
                    };

                    let (sql, values) = subscription.into_insert().build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, String, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .map_ok(|e| e.parse().unwrap())
                        .await?
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagUpsert {
                        subscription_id,
                        tag_id,
                        user_id: params.user_id,
                    };

                    let (sql, values) = subscription_tag
                        .into_insert()
                        .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_with(&sql, values).execute(&mut *tx).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn import_bookmarks(&self, params: ImportBookmarksParams) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            params.items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id = {
                    let tag = TagUpsert {
                        id: Uuid::new_v4(),
                        title: item.title,
                        user_id: params.user_id,
                    };

                    let (sql, values) = tag.clone().into_select().build_sqlx(SqliteQueryBuilder);

                    let row = sqlx::query_with(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await?;

                    match row {
                        Some(row) => row.get::<String, _>(0).parse().unwrap(),
                        _ => {
                            let id = tag.id;

                            let (sql, values) = tag.into_insert().build_sqlx(SqliteQueryBuilder);

                            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

                            id
                        }
                    }
                };

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let user_id = params.user_id;

                let bookmark_id = {
                    let bookmark = BookmarkUpsertParams {
                        url: link.parse().unwrap(),
                        bookmark: ProcessedBookmark {
                            title: item.title,
                            ..Default::default()
                        },
                        user_id,
                    };

                    let (sql, values) = bookmark.into_insert().build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, String, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .map_ok(|e| e.parse().unwrap())
                        .await?
                };

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = BookmarkTagUpsert {
                        bookmark_id,
                        tag_id,
                        user_id,
                    };

                    let (sql, values) = bookmark_tag.into_insert().build_sqlx(SqliteQueryBuilder);

                    sqlx::query_with(&sql, values).execute(&mut *tx).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
