use colette_core::scraper::{Error, SaveBookmarkData, SaveFeedData, ScraperRepository};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;

use crate::feed::create_feed_with_entries;

#[derive(Debug, Clone)]
pub struct PostgresScraperRepository {
    pool: PgPool,
}

impl PostgresScraperRepository {
    pub fn new(pool: PgPool) -> Self {
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

        let empty = data.feed.entries.is_empty();

        let feed_id = create_feed_with_entries(&mut tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if !empty {
            let fe_ids = {
                let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                    .build_sqlx(PostgresQueryBuilder);

                sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };

            {
                let insert_many = fe_ids
                    .into_iter()
                    .map(
                        |feed_entry_id| colette_sql::profile_feed_entry::InsertMany {
                            id: None,
                            feed_entry_id,
                        },
                    )
                    .collect::<Vec<_>>();

                let (sql, values) = colette_sql::profile_feed_entry::insert_many_for_all_profiles(
                    insert_many,
                    feed_id,
                )
                .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }

    async fn save_bookmark(&self, data: SaveBookmarkData) -> Result<(), Error> {
        let (sql, values) = colette_sql::bookmark::insert(
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}
