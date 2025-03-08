use colette_core::{
    FeedEntry,
    common::Transaction,
    feed_entry::{Error, FeedEntryById, FeedEntryFindParams, FeedEntryRepository},
};
use colette_model::{FeedEntryRow, feed_entries};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, FromQueryResult,
    sea_query::{Asterisk, Expr, Order, Query},
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteFeedEntryRepository {
    db: DatabaseConnection,
}

impl SqliteFeedEntryRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl FeedEntryRepository for SqliteFeedEntryRepository {
    async fn find_feed_entries(
        &self,
        params: FeedEntryFindParams,
    ) -> Result<Vec<FeedEntry>, Error> {
        let mut query = Query::select()
            .column(Asterisk)
            .from(feed_entries::Entity)
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.timestamp()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                );
            })
            .order_by(
                (feed_entries::Entity, feed_entries::Column::PublishedAt),
                Order::Desc,
            )
            .order_by(
                (feed_entries::Entity, feed_entries::Column::Id),
                Order::Desc,
            )
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let feed_entries =
            FeedEntryRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await
                .map(|e| e.into_iter().map(Into::into).collect())?;

        Ok(feed_entries)
    }

    async fn find_feed_entry_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<FeedEntryById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((feed_entries::Entity, feed_entries::Column::Id))
            .from(feed_entries::Entity)
            .and_where(
                Expr::col((feed_entries::Entity, feed_entries::Column::Id)).eq(id.to_string()),
            )
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(FeedEntryById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }
}
