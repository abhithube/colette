use chrono::{DateTime, Utc};
use colette_core::{
    common::{Findable, IdParams, Updatable},
    feed_entry::{Error, FeedEntryFindParams, FeedEntryRepository, FeedEntryUpdateData},
    FeedEntry,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;
use uuid::Uuid;

use super::smart_feed::build_case_statement;

#[derive(Debug, Clone)]
pub struct PostgresFeedEntryRepository {
    pool: PgPool,
}

impl PostgresFeedEntryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFeedEntryRepository {
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
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_as_with::<_, EntrySelect, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(FeedEntry::from).collect::<Vec<_>>())
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
            let count = {
                let (sql, values) =
                    crate::profile_feed_entry::update(params.id, params.profile_id, data.has_read)
                        .build_sqlx(PostgresQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&self.pool)
                    .await
                    .map(|e| e.rows_affected())
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
