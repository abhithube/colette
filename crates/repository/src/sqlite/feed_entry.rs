use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use deadpool_sqlite::{
    rusqlite::{self, Row},
    Pool,
};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;

use super::smart_feed::build_case_statement;

#[derive(Debug, Clone)]
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
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
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
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut entries: Vec<FeedEntry> = Vec::new();
            while let Some(row) = rows.next()? {
                entries.push(FeedEntrySelect::try_from(row).map(|e| e.0)?);
            }

            Ok(entries)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            if data.has_read.is_some() {
                let (sql, values) =
                    crate::user_feed_entry::update(params.id, params.user_id, data.has_read)
                        .build_rusqlite(SqliteQueryBuilder);

                let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

impl FeedEntryRepository for SqliteFeedEntryRepository {}

#[derive(Debug, Clone)]
struct FeedEntrySelect(FeedEntry);

impl TryFrom<&Row<'_>> for FeedEntrySelect {
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
            feed_id: value.get("user_feed_id")?,
        }))
    }
}
