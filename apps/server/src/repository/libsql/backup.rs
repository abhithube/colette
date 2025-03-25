use chrono::Utc;
use colette_core::backup::{BackupRepository, Error, ImportBookmarksData, ImportFeedsData};
use colette_query::{
    IntoInsert, IntoSelect,
    bookmark::BookmarkInsert,
    bookmark_tag::{BookmarkTagById, BookmarkTagInsert},
    feed::FeedInsert,
    subscription::SubscriptionInsert,
    subscription_tag::{SubscriptionTagById, SubscriptionTagInsert},
    tag::{TagInsert, TagSelectOne},
};
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlBackupRepository {
    conn: Connection,
}

impl LibsqlBackupRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl BackupRepository for LibsqlBackupRepository {
    async fn import_feeds(&self, data: ImportFeedsData) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = data
            .outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        #[derive(serde::Deserialize)]
        struct Row {
            id: Uuid,
        }

        while let Some((parent_id, outline)) = stack.pop() {
            if !outline.outline.is_empty() {
                let tag_id = {
                    let (sql, values) = TagSelectOne::Index {
                        title: &outline.text,
                        user_id: &data.user_id,
                    }
                    .into_select()
                    .build_libsql(SqliteQueryBuilder);

                    let mut stmt = tx.prepare(&sql).await?;
                    let mut rows = stmt.query(values.into_params()).await?;

                    match rows.next().await? {
                        Some(row) => {
                            let row = libsql::de::from_row::<Row>(&row)?;
                            row.id
                        }
                        _ => {
                            let id = Uuid::new_v4();

                            let (sql, values) = TagInsert {
                                id,
                                title: &outline.text,
                                user_id: &data.user_id,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                                upsert: true,
                            }
                            .into_insert()
                            .build_libsql(SqliteQueryBuilder);

                            let mut stmt = tx.prepare(&sql).await?;
                            stmt.execute(values.into_params()).await?;

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

                let (sql, values) = feed.into_insert().build_libsql(SqliteQueryBuilder);

                let mut stmt = self.conn.prepare(&sql).await?;
                let row = stmt.query_row(values.into_params()).await?;
                let row = libsql::de::from_row::<Row>(&row)?;

                let subscription_id = {
                    let (sql, values) = SubscriptionInsert {
                        id: Uuid::new_v4(),
                        title: &title,
                        feed_id: row.id,
                        user_id: &data.user_id,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        upsert: true,
                    }
                    .into_insert()
                    .build_libsql(SqliteQueryBuilder);

                    let mut stmt = self.conn.prepare(&sql).await?;
                    let row = stmt.query_row(values.into_params()).await?;
                    let row = libsql::de::from_row::<Row>(&row)?;

                    row.id
                };

                if let Some(tag_id) = parent_id {
                    let subscription_tag = SubscriptionTagInsert {
                        subscription_id,
                        tags: vec![SubscriptionTagById {
                            id: tag_id,
                            user_id: &data.user_id,
                        }],
                    };

                    let (sql, values) = subscription_tag
                        .into_insert()
                        .build_libsql(SqliteQueryBuilder);

                    let mut stmt = tx.prepare(&sql).await?;
                    stmt.execute(values.into_params()).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn import_bookmarks(&self, data: ImportBookmarksData) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            data.items.into_iter().map(|item| (None, item)).collect();

        #[derive(serde::Deserialize)]
        struct Row {
            id: Uuid,
        }

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id = {
                    let (sql, values) = TagSelectOne::Index {
                        title: &item.title,
                        user_id: &data.user_id,
                    }
                    .into_select()
                    .build_libsql(SqliteQueryBuilder);

                    let mut stmt = tx.prepare(&sql).await?;
                    let mut rows = stmt.query(values.into_params()).await?;

                    match rows.next().await? {
                        Some(row) => {
                            let row = libsql::de::from_row::<Row>(&row)?;
                            row.id
                        }
                        _ => {
                            let id = Uuid::new_v4();

                            let (sql, values) = TagInsert {
                                id,
                                title: &item.title,
                                user_id: &data.user_id,
                                created_at: Utc::now(),
                                updated_at: Utc::now(),
                                upsert: true,
                            }
                            .into_insert()
                            .build_libsql(SqliteQueryBuilder);

                            let mut stmt = tx.prepare(&sql).await?;
                            stmt.execute(values.into_params()).await?;

                            id
                        }
                    }
                };

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let bookmark_id = {
                    let (sql, values) = BookmarkInsert {
                        id: Uuid::new_v4(),
                        link: &link,
                        title: &item.title,
                        thumbnail_url: None,
                        published_at: None,
                        author: None,
                        archived_path: None,
                        user_id: &data.user_id,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                        upsert: true,
                    }
                    .into_insert()
                    .build_libsql(SqliteQueryBuilder);

                    let mut stmt = self.conn.prepare(&sql).await?;
                    let row = stmt.query_row(values.into_params()).await?;
                    let row = libsql::de::from_row::<Row>(&row)?;

                    row.id
                };

                if let Some(tag_id) = parent_id {
                    let bookmark_tag = BookmarkTagInsert {
                        bookmark_id,
                        tags: vec![BookmarkTagById {
                            id: tag_id,
                            user_id: &data.user_id,
                        }],
                    };

                    let (sql, values) = bookmark_tag.into_insert().build_libsql(SqliteQueryBuilder);

                    let mut stmt = tx.prepare(&sql).await?;
                    stmt.execute(values.into_params()).await?;
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
