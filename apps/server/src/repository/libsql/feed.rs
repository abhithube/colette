use chrono::DateTime;
use colette_core::{
    Feed,
    feed::{Error, FeedParams, FeedRepository},
};
use colette_query::{
    IntoInsert, IntoSelect,
    feed::FeedInsert,
    feed_entry::{FeedEntryInsert, FeedEntryInsertBatch},
};
use futures::{StreamExt, stream::BoxStream};
use libsql::Connection;
use sea_query::SqliteQueryBuilder;
use url::Url;
use uuid::Uuid;

use super::LibsqlBinder;

#[derive(Debug, Clone)]
pub struct LibsqlFeedRepository {
    conn: Connection,
}

impl LibsqlFeedRepository {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }
}

#[async_trait::async_trait]
impl FeedRepository for LibsqlFeedRepository {
    async fn query(&self, params: FeedParams) -> Result<Vec<Feed>, Error> {
        let (sql, values) = params.into_select().build_libsql(SqliteQueryBuilder);

        let mut stmt = self.conn.prepare(&sql).await?;
        let mut rows = stmt.query(values.into_params()).await?;

        let mut feeds = Vec::<Feed>::new();
        while let Some(row) = rows.next().await? {
            feeds.push(libsql::de::from_row::<FeedRow>(&row)?.into());
        }

        Ok(feeds)
    }

    async fn save(&self, data: &Feed) -> Result<(), Error> {
        let tx = self.conn.transaction().await?;

        let feed_id = {
            let feed = FeedInsert {
                id: data.id,
                link: data.link.as_str(),
                xml_url: data.xml_url.as_ref().map(|e| e.as_str()),
                title: &data.title,
                description: data.description.as_deref(),
                refreshed_at: data.refreshed_at,
            };

            let (sql, values) = feed.into_insert().build_libsql(SqliteQueryBuilder);

            #[derive(serde::Deserialize)]
            struct Row {
                id: Uuid,
            }

            let mut stmt = tx.prepare(&sql).await?;
            let row = stmt.query_row(values.into_params()).await?;

            libsql::de::from_row::<Row>(&row)?.id
        };

        if let Some(ref entries) = data.entries {
            let entries = entries.iter().map(|e| FeedEntryInsert {
                id: e.id,
                link: e.link.as_str(),
                title: &e.title,
                published_at: e.published_at,
                description: e.description.as_deref(),
                author: e.author.as_deref(),
                thumbnail_url: e.thumbnail_url.as_ref().map(|e| e.as_str()),
                feed_id,
            });

            let (sql, values) = FeedEntryInsertBatch(entries)
                .into_insert()
                .build_libsql(SqliteQueryBuilder);

            let mut stmt = tx.prepare(&sql).await?;
            stmt.execute(values.into_params()).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn stream(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let (sql, values) = FeedParams::default()
            .into_select()
            .build_libsql(SqliteQueryBuilder);

        let rows = self.conn.query(&sql, values.into_params()).await?;

        let stream = rows
            .into_stream()
            .map(|e| match e {
                Ok(row) => {
                    let row = libsql::de::from_row::<FeedRow>(&row)?;
                    Ok(row.xml_url.unwrap_or(row.link).parse::<Url>().unwrap())
                }
                Err(e) => Err(Error::Database(e)),
            })
            .boxed();

        Ok(stream)
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct FeedRow {
    pub(crate) id: Uuid,
    pub(crate) link: String,
    pub(crate) xml_url: Option<String>,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) refreshed_at: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link.parse().unwrap(),
            xml_url: value.xml_url.and_then(|e| e.parse().ok()),
            title: value.title,
            description: value.description,
            refreshed_at: value
                .refreshed_at
                .and_then(|e| DateTime::from_timestamp(e, 0)),
            entries: None,
        }
    }
}
