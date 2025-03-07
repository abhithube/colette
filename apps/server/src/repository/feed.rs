use colette_core::{
    Feed,
    feed::{Error, FeedFindParams, FeedRepository, FeedScrapedData},
};
use colette_model::{feeds, subscriptions};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    TransactionTrait, prelude::Expr, sea_query::Func,
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
        let feeds = feeds::Entity::find()
            .apply_if(params.id, |query, id| {
                query.filter(feeds::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(feeds::Column::Link.gt(cursor.link.to_string()))
            })
            .order_by_asc(feeds::Column::Link)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await
            .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(feeds)
    }

    async fn upsert_feed(&self, data: FeedScrapedData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let id = common::upsert_feed(
            &tx,
            data.feed.link,
            Some(data.url),
            data.feed.title,
            data.feed.description,
            data.feed.refreshed,
        )
        .await?;
        common::upsert_entries(&tx, data.feed.entries, id).await?;

        tx.commit().await?;

        Ok(id)
    }

    async fn stream_feed_urls(&self) -> Result<BoxStream<Result<Url, Error>>, Error> {
        let urls = feeds::Entity::find()
            .select_only()
            .expr_as(
                Func::coalesce([
                    Expr::col(feeds::Column::XmlUrl).into(),
                    Expr::col(feeds::Column::Link).into(),
                ]),
                "url",
            )
            .inner_join(subscriptions::Entity)
            .into_tuple::<String>()
            .stream(&self.db)
            .await?
            .map_ok(|e| e.parse().unwrap())
            .map_err(Error::Database)
            .boxed();

        Ok(urls)
    }
}
