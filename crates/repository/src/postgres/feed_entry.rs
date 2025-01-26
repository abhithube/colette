use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: Pool<Postgres>,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedEntryRepository {
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::user_feed_entry::select(
            params.id,
            params.user_id,
            params.feed_id,
            params.has_read,
            params.tags.as_deref(),
            // params.smart_feed_id,
            params.cursor,
            params.limit,
            // build_case_statement(),
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&self.pool)
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
        if data.has_read.is_some() {
            let (sql, values) =
                crate::user_feed_entry::update(params.id, params.user_id, data.has_read)
                    .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;
        }

        Ok(())
    }
}

impl FeedEntryRepository for PostgresFeedEntryRepository {}

#[derive(Debug, Clone)]
struct FeedEntrySelect(FeedEntry);

impl From<PgRow> for FeedEntrySelect {
    fn from(value: PgRow) -> Self {
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
