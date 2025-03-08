use colette_core::{
    Feed,
    feed::{Error, FeedFindParams, FeedRepository, FeedUpsertParams, StreamFeedUrlsParams},
};
use colette_model::{FeedRow, feeds};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, FromQueryResult, StreamTrait, TransactionTrait,
    sea_query::{Asterisk, Expr, Func, Order, Query},
};
use url::Url;
use uuid::Uuid;

use super::common;

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
        let mut query = Query::select()
            .column(Asterisk)
            .from(feeds::Entity)
            .apply_if(params.id, |query, id| {
                query.and_where(Expr::col((feeds::Entity, feeds::Column::Id)).eq(id.to_string()));
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::col((feeds::Entity, feeds::Column::Link))
                        .gt(Expr::val(cursor.link.to_string())),
                );
            })
            .order_by((feeds::Entity, feeds::Column::Link), Order::Asc)
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let feeds = FeedRow::find_by_statement(self.db.get_database_backend().build(&query))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(feeds)
    }

    async fn upsert_feed(&self, params: FeedUpsertParams) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let id = common::upsert_feed(
            &tx,
            params.feed.link,
            Some(params.url),
            params.feed.title,
            params.feed.description,
            params.feed.refreshed,
        )
        .await?;

        common::upsert_entries(&tx, params.feed.entries, id).await?;

        tx.commit().await?;

        Ok(id)
    }

    async fn stream_feed_urls(
        &self,
        _params: StreamFeedUrlsParams,
    ) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let query = Query::select()
            .expr(Func::coalesce([
                Expr::col(feeds::Column::XmlUrl).into(),
                Expr::col(feeds::Column::Link).into(),
            ]))
            .from(feeds::Entity)
            .to_owned();

        let urls = self
            .db
            .stream(self.db.get_database_backend().build(&query))
            .await?
            .map_ok(|e| e.try_get_by_index::<String>(0).unwrap().parse().unwrap())
            .map_err(Error::Database)
            .boxed();

        Ok(urls)
    }
}
