use colette_core::{
    backup::{BackupRepository, Error, ImportBookmarksData, ImportFeedsData},
    bookmark::BookmarkUpsertType,
    tag::TagUpsertType,
};
use colette_query::{
    IntoInsert, IntoSelect,
    bookmark::BookmarkInsert,
    bookmark_tag::{BookmarkTagById, BookmarkTagInsert},
    feed::FeedInsert,
    subscription::SubscriptionInsert,
    subscription_tag::{SubscriptionTagById, SubscriptionTagInsert},
    tag::{TagInsert, TagSelectOne},
};
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
    async fn import_feeds(&self, data: ImportFeedsData) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = data
            .outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                let tag_id = {
                    let (sql, values) = TagSelectOne::Index {
                        title: &outline.text,
                        user_id: data.user_id,
                    }
                    .into_select()
                    .build_sqlx(SqliteQueryBuilder);

                    let row = sqlx::query_with(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await?;

                    match row {
                        Some(row) => row.get::<Uuid, _>(0),
                        _ => {
                            let id = Uuid::new_v4();

                            let (sql, values) = TagInsert {
                                id,
                                title: &outline.text,
                                user_id: data.user_id,
                                upsert: Some(TagUpsertType::Title),
                            }
                            .into_insert()
                            .build_sqlx(SqliteQueryBuilder);

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

                let feed = FeedInsert {
                    id: Uuid::new_v4(),
                    link: &link,
                    xml_url: outline.xml_url.as_deref(),
                    title: &title,
                    description: None,
                    refreshed_at: None,
                };

                let (sql, values) = feed.into_insert().build_sqlx(SqliteQueryBuilder);

                let feed_id = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                    .fetch_one(&mut *tx)
                    .await?;

                let subscription_id = {
                    let (sql, values) = SubscriptionInsert {
                        id: Uuid::new_v4(),
                        title: &title,
                        feed_id,
                        user_id: data.user_id,
                        upsert: true,
                    }
                    .into_insert()
                    .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .await?
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagInsert {
                        subscription_id,
                        tags: vec![SubscriptionTagById {
                            id: tag_id,
                            user_id: data.user_id,
                        }],
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

    async fn import_bookmarks(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            data.items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id = {
                    let (sql, values) = TagSelectOne::Index {
                        title: &item.title,
                        user_id: data.user_id,
                    }
                    .into_select()
                    .build_sqlx(SqliteQueryBuilder);

                    let row = sqlx::query_with(&sql, values)
                        .fetch_optional(&mut *tx)
                        .await?;

                    match row {
                        Some(row) => row.get::<Uuid, _>(0),
                        _ => {
                            let id = Uuid::new_v4();

                            let (sql, values) = TagInsert {
                                id,
                                title: &item.title,
                                user_id: data.user_id,
                                upsert: Some(TagUpsertType::Title),
                            }
                            .into_insert()
                            .build_sqlx(SqliteQueryBuilder);

                            sqlx::query_with(&sql, values).execute(&mut *tx).await?;

                            id
                        }
                    }
                };

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let user_id = data.user_id;

                let bookmark_id = {
                    let (sql, values) = BookmarkInsert {
                        id: Uuid::new_v4(),
                        link: &link,
                        title: &item.title,
                        user_id: data.user_id,
                        upsert: Some(BookmarkUpsertType::Link),
                        ..Default::default()
                    }
                    .into_insert()
                    .build_sqlx(SqliteQueryBuilder);

                    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                        .fetch_one(&mut *tx)
                        .await?
                };

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = BookmarkTagInsert {
                        bookmark_id,
                        tags: vec![BookmarkTagById {
                            id: tag_id,
                            user_id,
                        }],
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
