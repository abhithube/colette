use colette_core::{
    FeedEntry,
    common::Transaction,
    feed_entry::{Error, FeedEntryById, FeedEntryFindParams, FeedEntryRepository},
};
use colette_model::feed_entries;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, QueryTrait, prelude::Expr,
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
        let feed_entries = feed_entries::Entity::find()
            .apply_if(params.id, |query, id| {
                query.filter(feed_entries::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        Expr::col((feed_entries::Entity, feed_entries::Column::PublishedAt)).into(),
                        Expr::col((feed_entries::Entity, feed_entries::Column::Id)).into(),
                    ])
                    .lt(Expr::tuple([
                        Expr::val(cursor.published_at.timestamp()).into(),
                        Expr::val(cursor.id.to_string()).into(),
                    ])),
                )
            })
            .order_by_desc(feed_entries::Column::PublishedAt)
            .order_by_desc(feed_entries::Column::Id)
            .limit(params.limit.map(|e| e as u64))
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

        let Some(id) = feed_entries::Entity::find()
            .select_only()
            .columns([feed_entries::Column::Id])
            .filter(feed_entries::Column::Id.eq(id.to_string()))
            .into_tuple::<String>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(FeedEntryById {
            id: id.parse().unwrap(),
        })
    }
}
