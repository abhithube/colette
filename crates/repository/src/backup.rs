use chrono::{DateTime, Utc};
use colette_core::backup::{BackupRepository, Error};
use colette_netscape::Item;
use colette_opml::Outline;
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
    async fn import_opml(&self, outlines: Vec<Outline>, user_id: Uuid) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut stack: Vec<(Option<Uuid>, Outline)> = outlines
            .into_iter()
            .map(|outline| (None, outline))
            .collect();

        while let Some((parent_id, outline)) = stack.pop() {
            let title = outline.title.unwrap_or(outline.text);

            if let Some(children) = outline.outline {
                let folder_id = sqlx::query_file_scalar!(
                    "queries/folders/upsert.sql",
                    title,
                    parent_id,
                    user_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                for child in children {
                    stack.push((Some(folder_id), child));
                }
            } else if let Some(link) = outline.html_url {
                let feed_id = sqlx::query_file_scalar!(
                    "queries/feeds/insert.sql",
                    link,
                    title,
                    outline.xml_url
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                sqlx::query_file_scalar!(
                    "queries/user_feeds/upsert.sql",
                    Option::<&str>::None,
                    parent_id,
                    feed_id,
                    user_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn import_netscape(&self, items: Vec<Item>, user_id: Uuid) -> Result<(), Error> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let mut stack: Vec<(Option<Uuid>, Item)> =
            items.into_iter().map(|item| (None, item)).collect();

        while let Some((parent_id, item)) = stack.pop() {
            if let Some(children) = item.item {
                let folder_id = sqlx::query_file_scalar!(
                    "queries/folders/upsert.sql",
                    item.title,
                    parent_id,
                    user_id
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                for child in children {
                    stack.push((Some(folder_id), child));
                }
            } else if let Some(link) = item.href {
                let bookmark_id = sqlx::query_file_scalar!(
                    "queries/bookmarks/insert.sql",
                    link,
                    item.title,
                    Option::<&str>::None,
                    Option::<DateTime<Utc>>::None,
                    Option::<&str>::None
                )
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

                sqlx::query_file_scalar!(
                    "queries/user_bookmarks/upsert.sql",
                    Option::<&str>::None,
                    Option::<&str>::None,
                    Option::<DateTime<Utc>>::None,
                    Option::<&str>::None,
                    parent_id,
                    bookmark_id,
                    user_id
                )
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
