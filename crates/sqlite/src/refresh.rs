use colette_core::refresh::{Error, FeedRefreshData, RefreshRepository};
use deadpool_sqlite::Pool;
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use crate::feed::create_feed_with_entries;

pub struct SqliteRefreshRepository {
    pub(crate) pool: Pool,
}

impl SqliteRefreshRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RefreshRepository for SqliteRefreshRepository {
    async fn refresh_feed(&self, data: FeedRefreshData) -> Result<(), Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let feed_id = create_feed_with_entries(&tx, data.url, data.feed)?;

            let fe_ids = {
                let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                    .build_rusqlite(SqliteQueryBuilder);

                let mut ids: Vec<i32> = Vec::new();

                let mut stmt = tx.prepare_cached(&sql)?;
                let mut rows = stmt.query(&*values.as_params())?;

                while let Some(row) = rows.next()? {
                    ids.push(row.get("id")?);
                }

                ids
            };

            let insert_many = fe_ids
                .into_iter()
                .map(
                    |feed_entry_id| colette_sql::profile_feed_entry::InsertMany {
                        id: Uuid::new_v4(),
                        feed_entry_id,
                    },
                )
                .collect::<Vec<_>>();

            {
                let (sql, values) = colette_sql::profile_feed_entry::insert_many_for_all_profiles(
                    insert_many,
                    feed_id,
                )
                .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            tx.commit()
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}
