use std::sync::Arc;

use chrono::{DateTime, Utc};
use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use sea_query::SqliteQueryBuilder;
use uuid::Uuid;
use worker::D1Database;

use super::{smart_feed::build_case_statement, D1Binder};

#[derive(Clone)]
pub struct D1FeedEntryRepository {
    db: Arc<D1Database>,
}

impl D1FeedEntryRepository {
    pub fn new(db: Arc<D1Database>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for D1FeedEntryRepository {
    type Params = FeedEntryFindParams;
    type Output = Result<Vec<FeedEntry>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::profile_feed_entry::select(
            params.id,
            params.profile_id,
            params.feed_id,
            params.has_read,
            params.tags.as_deref(),
            params.smart_feed_id,
            params.cursor,
            params.limit,
            build_case_statement(),
        )
        .build_d1(SqliteQueryBuilder);

        let result = super::all(&self.db, sql, values)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        result
            .results::<EntrySelect>()
            .map(|e| e.into_iter().map(FeedEntry::from).collect())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for D1FeedEntryRepository {
    type Params = IdParams;
    type Data = FeedEntryUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.has_read.is_some() {
            let (sql, values) = crate::profile_feed_entry::update(
                params.id,
                params.profile_id,
                data.has_read,
            )
            .build_d1(SqliteQueryBuilder);

            let result = super::run(&self.db, sql, values)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            let meta = result.meta().unwrap().unwrap();

            if meta.changes.is_none_or(|e| e == 0) {
                return Err(Error::NotFound(params.id));
            }
        }

        Ok(())
    }
}

impl FeedEntryRepository for D1FeedEntryRepository {}

#[derive(Debug, Clone, serde::Deserialize)]
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