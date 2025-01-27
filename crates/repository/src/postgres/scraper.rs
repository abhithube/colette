use colette_core::scraper::{Error, SaveBookmarkData, SaveFeedData, ScraperRepository};
use sqlx::{Pool, Postgres};

use super::feed::{create_feed_with_entries, link_entries_to_users};

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
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = create_feed_with_entries(&mut tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        link_entries_to_users(&mut tx, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error> {
        crate::bookmark::insert(
            &self.pool,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .await
        .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}
