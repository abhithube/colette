use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use deadpool_postgres::{GenericClient, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::Row;
use uuid::Uuid;

use crate::smart_feed::build_case_statement;

pub struct PostgresFeedEntryRepository {
    pub(crate) pool: Pool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedEntryRepository {
    type Params = IdParams;
    type Output = Result<FeedEntry, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find_by_id(&client, params).await
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<FeedEntry, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.has_read.is_some() {
            let count = {
                let (sql, values) = colette_sql::profile_feed_entry::update(
                    params.id,
                    params.profile_id,
                    data.has_read,
                )
                .build_postgres(PostgresQueryBuilder);

                tx.execute(&sql, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let entry = find_by_id(&tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(entry)
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for PostgresFeedEntryRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<FeedEntryFindManyFilters>,
    ) -> Result<Vec<FeedEntry>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone)]
struct EntrySelect(FeedEntry);

impl From<&Row> for EntrySelect {
    fn from(value: &Row) -> Self {
        Self(FeedEntry {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            published_at: value.get("published_at"),
            description: value.get("description"),
            author: value.get("author"),
            thumbnail_url: value.get("thumbnail_url"),
            has_read: value.get("has_read"),
            feed_id: value.get("profile_feed_id"),
        })
    }
}

async fn find<C: GenericClient>(
    client: &C,
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
    .build_postgres(PostgresQueryBuilder);

    client
        .query(&sql, &values.as_params())
        .await
        .map(|e| {
            e.into_iter()
                .map(|e| EntrySelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id<C: GenericClient>(client: &C, params: IdParams) -> Result<FeedEntry, Error> {
    let mut feed_entries =
        find(client, Some(params.id), params.profile_id, None, None, None).await?;
    if feed_entries.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(feed_entries.swap_remove(0))
}
