use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Uuid,
    },
    SqliteExecutor, SqlitePool,
};

use crate::smart_feed::build_case_statement;

pub struct SqliteFeedEntryRepository {
    pub(crate) pool: SqlitePool,
}

impl SqliteFeedEntryRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Output = Result<FeedEntry, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<FeedEntry, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.has_read.is_some() {
            let result = {
                let (sql, values) = colette_sql::profile_feed_entry::update(
                    params.id,
                    params.profile_id,
                    data.has_read,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if result.rows_affected() == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let entry = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(entry)
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Vec<FeedEntry>, Error> {
        find(&self.pool, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct EntrySelect {
    id: Uuid,
    link: String,
    title: String,
    published_at: DateTime<Utc>,
    description: Option<String>,
    author: Option<String>,
    thumbnail_url: Option<String>,
    has_read: bool,
    profile_feed_id: Uuid,
}

impl From<EntrySelect> for colette_core::FeedEntry {
    fn from(value: EntrySelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            published_at: value.published_at,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail_url,
            has_read: value.has_read,
            feed_id: value.profile_feed_id,
        }
    }
}

async fn find(
    executor: impl SqliteExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> Result<Vec<FeedEntry>, Error> {
    let mut feed_id: Option<Uuid> = None;
    let mut smart_feed_id: Option<Uuid> = None;
    let mut has_read: Option<bool> = None;
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        feed_id = filters.feed_id;
        smart_feed_id = filters.smart_feed_id;
        has_read = filters.has_read;
        tags = filters.tags;
    }

    let (sql, values) = colette_sql::profile_feed_entry::select(
        id,
        profile_id,
        feed_id,
        has_read,
        tags.as_deref(),
        smart_feed_id,
        cursor,
        limit,
        build_case_statement(),
    )
    .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with::<_, EntrySelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(FeedEntry::from).collect())
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(
    executor: impl SqliteExecutor<'_>,
    params: IdParams,
) -> Result<FeedEntry, Error> {
    let mut feed_entries = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if feed_entries.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feed_entries.swap_remove(0))
}
