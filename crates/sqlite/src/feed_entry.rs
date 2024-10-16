use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{
        Cursor, Error, FeedEntryFindManyFilters, FeedEntryRepository, FeedEntryUpdateData,
    },
    FeedEntry,
};
use deadpool_sqlite::Pool;
use rusqlite::{Connection, Row};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

use crate::smart_feed::build_case_statement;

pub struct SqliteFeedEntryRepository {
    pool: Pool,
}

impl SqliteFeedEntryRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Output = Result<FeedEntry, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find_by_id(conn, params.id, params.profile_id))
            .await
            .unwrap()
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<FeedEntry, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if data.has_read.is_some() {
                let count = {
                    let (sql, values) = colette_sql::profile_feed_entry::update(
                        params.id,
                        params.profile_id,
                        data.has_read,
                    )
                    .build_rusqlite(SqliteQueryBuilder);

                    tx.prepare_cached(&sql)?.execute(&*values.as_params())?
                };
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            let entry = find_by_id(&tx, params.id, params.profile_id)?;

            tx.commit()?;

            Ok(entry)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
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
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find(conn, None, profile_id, limit, cursor, filters))
            .await
            .unwrap()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct EntrySelect(FeedEntry);

impl TryFrom<&Row<'_>> for EntrySelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(FeedEntry {
            id: value.get("id")?,
            link: value.get("link")?,
            title: value.get("title")?,
            published_at: value.get("published_at")?,
            description: value.get("description")?,
            author: value.get("author")?,
            thumbnail_url: value.get("thumbnail_url")?,
            has_read: value.get("has_read")?,
            feed_id: value.get("profile_feed_id")?,
        }))
    }
}

fn find(
    conn: &Connection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<FeedEntryFindManyFilters>,
) -> rusqlite::Result<Vec<FeedEntry>> {
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
    .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(&*values.as_params())?;

    let mut entries: Vec<FeedEntry> = Vec::new();
    while let Some(row) = rows.next()? {
        entries.push(EntrySelect::try_from(row).map(|e| e.0)?);
    }

    Ok(entries)
}

fn find_by_id(conn: &Connection, id: Uuid, profile_id: Uuid) -> rusqlite::Result<FeedEntry> {
    let mut entries = find(conn, Some(id), profile_id, None, None, None)?;
    if entries.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(entries.swap_remove(0))
}
