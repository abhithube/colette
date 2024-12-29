use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;

use super::smart_feed::build_case_statement;

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: Pool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedEntryRepository {
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::user_feed_entry::select(
            params.id,
            params.user_id,
            params.feed_id,
            params.has_read,
            params.tags.as_deref(),
            params.smart_feed_id,
            params.cursor,
            params.limit,
            build_case_statement(),
        )
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query(&stmt, &values.as_params())
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| FeedEntrySelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.has_read.is_some() {
            let count = {
                let (sql, values) =
                    crate::user_feed_entry::update(params.id, params.user_id, data.has_read)
                        .build_postgres(PostgresQueryBuilder);

                let stmt = client
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                client
                    .execute(&stmt, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        Ok(())
    }
}

impl FeedEntryRepository for PostgresFeedEntryRepository {}

#[derive(Debug, Clone)]
struct FeedEntrySelect(FeedEntry);

impl From<Row> for FeedEntrySelect {
    fn from(value: Row) -> Self {
        Self(FeedEntry {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            published_at: value.get("published_at"),
            description: value.get("description"),
            author: value.get("author"),
            thumbnail_url: value.get("thumbnail_url"),
            has_read: value.get("has_read"),
            feed_id: value.get("user_feed_id"),
        })
    }
}
