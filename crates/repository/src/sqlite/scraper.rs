use colette_core::scraper::{Error, SaveBookmarkData, SaveFeedData, ScraperRepository};
use deadpool_sqlite::{rusqlite, Pool};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use super::feed::{create_feed_with_entries, link_entries_to_users};

#[derive(Debug, Clone)]
pub struct SqliteScraperRepository {
    pool: Pool,
}

impl SqliteScraperRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScraperRepository for SqliteScraperRepository {
    async fn save_feed(&self, data: SaveFeedData) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let feed_id = create_feed_with_entries(&tx, data.url, data.feed)?;

            link_entries_to_users(&tx, feed_id)?;

            tx.commit()
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::bookmark::insert(
                Some(Uuid::new_v4()),
                data.url,
                data.bookmark.title,
                data.bookmark.thumbnail.map(String::from),
                data.bookmark.published,
                data.bookmark.author,
            )
            .build_rusqlite(SqliteQueryBuilder);

            conn.prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))?;

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}
