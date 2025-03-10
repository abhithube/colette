use chrono::{DateTime, Utc};
use colette_core::{
    Feed,
    feed::{Error, FeedFindParams, FeedRepository, FeedStreamUrlsParams, FeedUpsertParams},
};
use colette_query::{IntoInsert, IntoSelect, feed::FeedUpsert, feed_entry::FeedEntryUpsert};
use futures::{
    StreamExt,
    stream::{self, BoxStream},
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{Pool, Sqlite};
use url::Url;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteFeedRepository {
    pool: Pool<Sqlite>,
}

impl SqliteFeedRepository {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn find_feeds(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn upsert_feed(&self, params: FeedUpsertParams) -> Result<Uuid, Error> {
        let mut tx = self.pool.begin().await?;

        let feed = FeedUpsert {
            id: Uuid::new_v4(),
            link: params.feed.link,
            xml_url: Some(params.url),
            title: params.feed.title,
            description: params.feed.description,
            refreshed_at: params.feed.refreshed,
        };

        let (sql, values) = feed.into_insert().build_sqlx(SqliteQueryBuilder);

        let id = sqlx::query_scalar_with::<_, String, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await?
            .parse()
            .unwrap();

        let entries = params
            .feed
            .entries
            .into_iter()
            .map(|e| FeedEntryUpsert {
                id: Uuid::new_v4(),
                link: e.link,
                title: e.title,
                published_at: e.published,
                description: e.description,
                author: e.author,
                thumbnail_url: e.thumbnail,
                feed_id: id,
            })
            .collect::<Vec<_>>();

        let (sql, values) = entries.into_insert().build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *tx).await?;

        tx.commit().await?;

        Ok(id)
    }

    async fn stream_feed_urls(
        &self,
        params: FeedStreamUrlsParams,
    ) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let (sql, values) = params.into_select().build_sqlx(SqliteQueryBuilder);

        let urls = sqlx::query_scalar_with::<_, String, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(stream::iter(urls.into_iter().map(|e| Ok(e.parse().unwrap()))).boxed())
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct FeedRow {
    pub(crate) id: String,
    pub(crate) link: String,
    pub(crate) xml_url: Option<String>,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) refreshed_at: Option<DateTime<Utc>>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            xml_url: value.xml_url.and_then(|e| e.parse().ok()),
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at,
        }
    }
}
