use colette_core::scraper::{Error, SaveFeedData, ScraperRepository};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use uuid::Uuid;

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

        let feed_id = {
            let link = data.feed.link.to_string();
            let url = if data.url == link {
                None
            } else {
                Some(data.url)
            };

            let (sql, values) = colette_sql::feed::insert(link, data.feed.title, url)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        {
            let insert_many = data
                .feed
                .entries
                .into_iter()
                .map(|e| colette_sql::feed_entry::InsertMany {
                    link: e.link.to_string(),
                    title: e.title,
                    published_at: e.published,
                    description: e.description,
                    author: e.author,
                    thumbnail_url: e.thumbnail.map(String::from),
                })
                .collect::<Vec<_>>();

            let (sql, values) = colette_sql::feed_entry::insert_many(insert_many, feed_id)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

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
                        id: Uuid::new_v4(),
                        feed_entry_id,
                    },
                )
                .collect::<Vec<_>>();

            let (sql, values) =
                colette_sql::profile_feed_entry::insert_many_for_all_profiles(insert_many, feed_id)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))
    }
}
