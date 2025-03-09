use colette_core::{
    Feed,
    feed::{Error, FeedFindParams, FeedRepository, FeedStreamUrlsParams, FeedUpsertParams},
};
use colette_query::{IntoInsert, IntoSelect, feed::FeedUpsert, feed_entry::FeedEntryUpsert};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, FromQueryResult, StreamTrait, TransactionTrait,
};
use url::Url;
use uuid::Uuid;

use super::common::parse_timestamp;

#[derive(Debug, Clone)]
pub struct SqliteFeedRepository {
    db: DatabaseConnection,
}

impl SqliteFeedRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedRepository for SqliteFeedRepository {
    async fn find_feeds(&self, params: FeedFindParams) -> Result<Vec<Feed>, Error> {
        let feeds =
            FeedRow::find_by_statement(self.db.get_database_backend().build(&params.into_select()))
                .all(&self.db)
                .await
                .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(feeds)
    }

    async fn upsert_feed(&self, params: FeedUpsertParams) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let feed = FeedUpsert {
            id: Uuid::new_v4(),
            link: params.feed.link,
            xml_url: Some(params.url),
            title: params.feed.title,
            description: params.feed.description,
            refreshed_at: params.feed.refreshed,
        };

        let id = tx
            .query_one(self.db.get_database_backend().build(&feed.into_insert()))
            .await?
            .unwrap()
            .try_get_by_index::<String>(0)
            .unwrap()
            .parse::<Uuid>()
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

        tx.execute(self.db.get_database_backend().build(&entries.into_insert()))
            .await?;

        tx.commit().await?;

        Ok(id)
    }

    async fn stream_feed_urls(
        &self,
        params: FeedStreamUrlsParams,
    ) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let urls = self
            .db
            .stream(self.db.get_database_backend().build(&params.into_select()))
            .await?
            .map_ok(|e| e.try_get_by_index::<String>(0).unwrap().parse().unwrap())
            .map_err(Error::Database)
            .boxed();

        Ok(urls)
    }
}

#[derive(sea_orm::FromQueryResult)]
pub(crate) struct FeedRow {
    pub(crate) id: String,
    pub(crate) link: String,
    pub(crate) xml_url: Option<String>,
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) refreshed_at: Option<i32>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            link: value.link.parse().unwrap(),
            xml_url: value.xml_url.and_then(|e| e.parse().ok()),
            title: value.title,
            description: value.description,
            refreshed_at: value.refreshed_at.and_then(parse_timestamp),
        }
    }
}
