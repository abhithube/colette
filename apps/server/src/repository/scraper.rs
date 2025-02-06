use colette_core::scraper::{Error, SaveBookmarkData, SaveFeedData, ScraperRepository};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::common;

#[derive(Debug, Clone)]
pub struct PostgresScraperRepository {
    pool: Pool<Postgres>,
}

impl PostgresScraperRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScraperRepository for PostgresScraperRepository {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let feed_id = common::insert_feed_with_entries(&mut *tx, data.url, data.feed).await?;

        sqlx::query_file!("queries/user_feed_entries/insert_many.sql", feed_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error> {
        sqlx::query_file_scalar!(
            "queries/bookmarks/upsert.sql",
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
            Option::<Uuid>::None,
            data.user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
