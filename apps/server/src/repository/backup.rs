use chrono::{DateTime, Utc};
use colette_core::backup::{BackupRepository, Error};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBackupRepository {
    pool: Pool<Postgres>,
}

impl PostgresBackupRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl BackupRepository for PostgresBackupRepository {
    async fn import_feeds(
        &self,
        outlines: Vec<colette_opml::Outline>,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_opml::Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if !outline.outline.is_empty() {
                let tag_id = sqlx::query_file_scalar!("queries/tags/upsert.sql", title, user_id)
                    .fetch_one(&mut *tx)
                    .await?;

                for child in outline.outline {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let feed_id =
                    sqlx::query_file_scalar!("queries/feeds/insert.sql", link, outline.xml_url)
                        .fetch_one(&mut *tx)
                        .await?;

                let uf_id = sqlx::query_file_scalar!(
                    "queries/user_feeds/upsert.sql",
                    Option::<&str>::None,
                    feed_id,
                    user_id
                )
                .fetch_one(&mut *tx)
                .await?;

                sqlx::query_file_scalar!(
                    "queries/user_feed_tags/insert.sql",
                    uf_id,
                    parent_id,
                    user_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }

    async fn import_bookmarks(
        &self,
        items: Vec<colette_netscape::Item>,
        user_id: Uuid,
    ) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let mut stack: Vec<(Option<Uuid>, colette_netscape::Item)> =
            items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if !item.item.is_empty() {
                let tag_id =
                    sqlx::query_file_scalar!("queries/tags/upsert.sql", item.title, user_id)
                        .fetch_one(&mut *tx)
                        .await?;

                for child in item.item {
                    stack.push((Some(tag_id), child));
                }
            } else if let Some(link) = item.href {
                let bookmark_id = sqlx::query_file_scalar!(
                    "queries/bookmarks/upsert.sql",
                    link,
                    item.title,
                    Option::<&str>::None,
                    Option::<DateTime<Utc>>::None,
                    Option::<&str>::None,
                    user_id
                )
                .fetch_one(&mut *tx)
                .await?;

                sqlx::query_file_scalar!(
                    "queries/bookmark_tags/insert.sql",
                    bookmark_id,
                    parent_id,
                    user_id
                )
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
