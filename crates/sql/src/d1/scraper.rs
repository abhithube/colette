use std::sync::Arc;

use colette_core::scraper::{Error, SaveBookmarkData, SaveFeedData, ScraperRepository};
use sea_query::SqliteQueryBuilder;
use worker::D1Database;

use super::{
    feed::{create_feed_with_entries, link_entries_to_profiles},
    D1Binder,
};

#[derive(Clone)]
pub struct D1ScraperRepository {
    db: Arc<D1Database>,
}

impl D1ScraperRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl ScraperRepository for D1ScraperRepository {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error> {
        let feed_id = create_feed_with_entries(&self.db, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        link_entries_to_profiles(&self.db, feed_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error> {
        let (sql, values) = crate::bookmark::insert(
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_d1(SqliteQueryBuilder);

        super::run(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}
