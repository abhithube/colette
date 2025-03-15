use chrono::{DateTime, Utc};
use colette_core::{
    Feed,
    feed::{Error, FeedFindParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::{FeedInsert, FeedSelect},
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
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
    async fn find(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error> {
        let (sql, values) = FeedSelect {
            id: params.id,
            cursor: params.cursor.as_deref(),
            limit: params.limit,
        }
        .into_select()
        .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn save(&self, data: &Feed) -> Result<(), Error> {
        let mut tx = self.pool.begin().await?;

        let feed = FeedInsert {
            id: data.id,
            link: data.link.as_str(),
            xml_url: data.xml_url.as_ref().map(|e| e.as_str()),
            title: &data.title,
            description: data.description.as_deref(),
            refreshed_at: data.refreshed_at,
        };

        let (sql, values) = feed.into_insert().build_sqlx(SqliteQueryBuilder);

        let id = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_one(&mut *tx)
            .await?;

        if let Some(ref entries) = data.entries {
            let entries = entries.iter().map(|e| FeedEntryInsert {
                id: e.id,
                link: e.link.as_str(),
                title: &e.title,
                published_at: e.published_at,
                description: e.description.as_deref(),
                author: e.author.as_deref(),
                thumbnail_url: e.thumbnail_url.as_ref().map(|e| e.as_str()),
                feed_id: id,
            });

            let (sql, values) = FeedEntryInsertBatch(entries)
                .into_insert()
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values).execute(&mut *tx).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let (sql, values) = FeedSelect::default()
            .into_select()
            .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as_with::<_, FeedRow, _>(&sql, values)
            .fetch_all(&self.pool)
            .await?;

        Ok(stream::iter(
            rows.into_iter()
                .map(|e| Ok(e.xml_url.unwrap_or(e.link).parse().unwrap())),
        )
        .boxed())
    }
}

#[derive(sqlx::FromRow)]
pub(crate) struct FeedRow {
    pub(crate) id: Uuid,
    pub(crate) link: String,
    pub(crate) xml_url: Option<String>,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) refreshed_at: Option<DateTime<Utc>>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link.parse().unwrap(),
            xml_url: value.xml_url.and_then(|e| e.parse().ok()),
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at,
            entries: None,
        }
    }
}
