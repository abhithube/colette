use colette_core::refresh::{Error, FeedRefreshData, RefreshRepository};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::types::Type;
use uuid::Uuid;

use crate::feed::create_feed_with_entries;

pub struct PostgresRefreshRepository {
    pub(crate) pool: Pool,
}

impl PostgresRefreshRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RefreshRepository for PostgresRefreshRepository {
    async fn refresh_feed(&self, data: FeedRefreshData) -> Result<(), Error> {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let feed_id = create_feed_with_entries(&tx, data.url, data.feed)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let fe_ids = {
            let (sql, values) = colette_sql::feed_entry::select_many_by_feed_id(feed_id)
                .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let mut ids: Vec<i32> = Vec::new();
            for row in tx
                .query(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                ids.push(row.get("id"));
            }

            ids
        };

        let mut types: Vec<Type> = Vec::new();
        let insert_many = fe_ids
            .into_iter()
            .map(|feed_entry_id| {
                types.push(Type::UUID);
                types.push(Type::INT4);
                types.push(Type::INT4);

                colette_sql::profile_feed_entry::InsertMany {
                    id: Uuid::new_v4(),
                    feed_entry_id,
                }
            })
            .collect::<Vec<_>>();

        {
            let (sql, values) =
                colette_sql::profile_feed_entry::insert_many_for_all_profiles(insert_many, feed_id)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_typed_cached(&sql, &types)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
