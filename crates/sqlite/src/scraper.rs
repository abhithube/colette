use colette_core::scraper::{Error, SaveFeedData, ScraperRepository};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use uuid::Uuid;

use crate::feed::create_feed_with_entries;

pub struct SqliteScraperRepository {
    pool: SqlitePool,
}

impl SqliteScraperRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ScraperRepository for SqliteScraperRepository {
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
                    .build_sqlx(SqliteQueryBuilder);

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
                            id: Some(Uuid::new_v4()),
                            feed_entry_id,
                        },
                    )
                    .collect::<Vec<_>>();

                let (sql, values) = colette_sql::profile_feed_entry::insert_many_for_all_profiles(
                    insert_many,
                    feed_id,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;
            }
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
