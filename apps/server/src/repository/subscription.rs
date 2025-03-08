use std::collections::HashMap;

use colette_core::{
    Subscription,
    common::Transaction,
    subscription::{
        Error, SubscriptionById, SubscriptionCreateParams, SubscriptionDeleteParams,
        SubscriptionEntryUpdateParams, SubscriptionFindByIdParams, SubscriptionFindParams,
        SubscriptionRepository, SubscriptionTagsLinkParams, SubscriptionUpdateParams,
    },
};
use colette_model::{
    SubscriptionRow, SubscriptionTagRow, SubscriptionWithTagsAndCount, feed_entries, feeds,
    read_entries, subscription_tags, subscriptions, tags,
};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult,
    sea_query::{Alias, Expr, Func, OnConflict, Order, Query},
};

#[derive(Debug, Clone)]
pub struct SqliteSubscriptionRepository {
    db: DatabaseConnection,
}

impl SqliteSubscriptionRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl SubscriptionRepository for SqliteSubscriptionRepository {
    async fn find_subscriptions(
        &self,
        params: SubscriptionFindParams,
    ) -> Result<Vec<Subscription>, Error> {
        let mut query = Query::select()
            .columns([
                (subscriptions::Entity, subscriptions::Column::Id),
                (subscriptions::Entity, subscriptions::Column::Title),
                (subscriptions::Entity, subscriptions::Column::UserId),
                (subscriptions::Entity, subscriptions::Column::CreatedAt),
                (subscriptions::Entity, subscriptions::Column::UpdatedAt),
            ])
            .columns([
                (feeds::Entity, feeds::Column::Link),
                (feeds::Entity, feeds::Column::XmlUrl),
                (feeds::Entity, feeds::Column::Description),
                (feeds::Entity, feeds::Column::RefreshedAt),
            ])
            .expr_as(
                Expr::col((feeds::Entity, feeds::Column::Id)),
                Alias::new("feed_id"),
            )
            .expr_as(
                Expr::col((feeds::Entity, feeds::Column::Title)),
                Alias::new("feed_title"),
            )
            .from(subscriptions::Entity)
            .inner_join(
                feeds::Entity,
                Expr::col((feeds::Entity, feeds::Column::Id)).eq(Expr::col((
                    subscriptions::Entity,
                    subscriptions::Column::FeedId,
                ))),
            )
            .apply_if(params.id, |query, id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                        .eq(id.to_string()),
                );
            })
            .apply_if(params.user_id, |query, user_id| {
                query.and_where(
                    Expr::col((subscriptions::Entity, subscriptions::Column::UserId))
                        .eq(user_id.to_string()),
                );
            })
            .apply_if(params.tags, |query, tags| {
                query.and_where(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(subscription_tags::Entity)
                        .and_where(
                            Expr::col((
                                subscription_tags::Entity,
                                subscription_tags::Column::SubscriptionId,
                            ))
                            .eq(Expr::col((
                                subscriptions::Entity,
                                subscriptions::Column::Id,
                            ))),
                        )
                        .and_where(
                            Expr::col((
                                subscription_tags::Entity,
                                subscription_tags::Column::TagId,
                            ))
                            .is_in(tags.into_iter().map(String::from)),
                        )
                        .to_owned(),
                ));
            })
            .apply_if(params.cursor, |query, cursor| {
                query.and_where(
                    Expr::tuple([
                        Expr::col(subscriptions::Column::Title).into(),
                        Expr::col(subscriptions::Column::Id).into(),
                    ])
                    .gt(Expr::tuple([
                        Expr::value(cursor.title),
                        Expr::value(cursor.id.to_string()),
                    ])),
                );
            })
            .order_by(
                (subscriptions::Entity, subscriptions::Column::Title),
                Order::Asc,
            )
            .order_by(
                (subscriptions::Entity, subscriptions::Column::Id),
                Order::Asc,
            )
            .to_owned();

        if let Some(limit) = params.limit {
            query.limit(limit as u64);
        }

        let subscription_rows =
            SubscriptionRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await?;

        let query = Query::select()
            .column((
                subscription_tags::Entity,
                subscription_tags::Column::SubscriptionId,
            ))
            .columns([
                (tags::Entity, tags::Column::Id),
                (tags::Entity, tags::Column::Title),
                (tags::Entity, tags::Column::CreatedAt),
                (tags::Entity, tags::Column::UpdatedAt),
                (tags::Entity, tags::Column::UserId),
            ])
            .from(subscription_tags::Entity)
            .inner_join(
                tags::Entity,
                Expr::col((tags::Entity, tags::Column::Id)).eq(Expr::col((
                    subscription_tags::Entity,
                    subscription_tags::Column::TagId,
                ))),
            )
            .and_where(
                Expr::col((
                    subscription_tags::Entity,
                    subscription_tags::Column::SubscriptionId,
                ))
                .is_in(subscription_rows.iter().map(|e| e.id.as_str())),
            )
            .order_by((tags::Entity, tags::Column::Title), Order::Asc)
            .to_owned();

        let tag_rows =
            SubscriptionTagRow::find_by_statement(self.db.get_database_backend().build(&query))
                .all(&self.db)
                .await?;

        let mut tag_row_map = HashMap::<String, Vec<SubscriptionTagRow>>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id.clone())
                .or_default()
                .push(row);
        }

        let query = Query::select()
            .column((subscriptions::Entity, subscriptions::Column::Id))
            .expr(Func::count(Expr::col((
                feed_entries::Entity,
                feed_entries::Column::Id,
            ))))
            .from(feed_entries::Entity)
            .inner_join(
                subscriptions::Entity,
                Expr::col((subscriptions::Entity, subscriptions::Column::FeedId)).eq(Expr::col((
                    feed_entries::Entity,
                    feed_entries::Column::FeedId,
                ))),
            )
            .and_where(
                Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                    .is_in(subscription_rows.iter().map(|e| e.id.as_str())),
            )
            .and_where(
                Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(read_entries::Entity)
                        .and_where(
                            Expr::col((read_entries::Entity, read_entries::Column::FeedEntryId))
                                .eq(Expr::col((feed_entries::Entity, feed_entries::Column::Id))),
                        )
                        .and_where(
                            Expr::col((read_entries::Entity, read_entries::Column::SubscriptionId))
                                .eq(Expr::col((
                                    subscriptions::Entity,
                                    subscriptions::Column::Id,
                                ))),
                        )
                        .to_owned(),
                )
                .not(),
            )
            .group_by_col((subscriptions::Entity, subscriptions::Column::Id))
            .to_owned();

        let mut unread_count_map = HashMap::<String, i64>::new();

        let results = self
            .db
            .query_all(self.db.get_database_backend().build(&query))
            .await?;

        for row in results {
            unread_count_map
                .entry(row.try_get_by_index(0).unwrap())
                .insert_entry(row.try_get_by_index(1).unwrap());
        }

        let subscriptions = subscription_rows
            .into_iter()
            .map(|subscription| {
                SubscriptionWithTagsAndCount {
                    tags: tag_row_map.remove(&subscription.id),
                    unread_count: unread_count_map.remove(&subscription.id),
                    subscription,
                }
                .into()
            })
            .collect();

        Ok(subscriptions)
    }

    async fn find_subscription_by_id(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionFindByIdParams,
    ) -> Result<SubscriptionById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::select()
            .column((subscriptions::Entity, subscriptions::Column::Id))
            .column((subscriptions::Entity, subscriptions::Column::UserId))
            .from(subscriptions::Entity)
            .and_where(
                Expr::col((subscriptions::Entity, subscriptions::Column::Id))
                    .eq(params.id.to_string()),
            )
            .to_owned();

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&query))
            .await?
        else {
            return Err(Error::NotFound(params.id));
        };

        Ok(SubscriptionById {
            id: result
                .try_get_by_index::<String>(0)
                .unwrap()
                .parse()
                .unwrap(),
            user_id: result
                .try_get_by_index::<String>(1)
                .unwrap()
                .parse()
                .unwrap(),
        })
    }

    async fn create_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionCreateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::insert()
            .into_table(subscriptions::Entity)
            .columns([
                subscriptions::Column::Id,
                subscriptions::Column::Title,
                subscriptions::Column::FeedId,
                subscriptions::Column::UserId,
            ])
            .values_panic([
                params.id.to_string().into(),
                params.title.clone().into(),
                params.feed_id.to_string().into(),
                params.user_id.to_string().into(),
            ])
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(params.feed_id),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn update_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.title.is_none() {
            return Ok(());
        }

        let mut query = Query::update()
            .table(subscriptions::Entity)
            .and_where(Expr::col(subscriptions::Column::Id).eq(params.id.to_string()))
            .to_owned();

        if let Some(title) = params.title {
            query.value(subscriptions::Column::Title, title);
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn delete_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(subscriptions::Entity)
            .and_where(Expr::col(subscriptions::Column::Id).eq(params.id.to_string()))
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionTagsLinkParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let query = Query::delete()
            .from_table(subscription_tags::Entity)
            .and_where(
                Expr::col(subscription_tags::Column::SubscriptionId)
                    .eq(params.subscription_id.to_string()),
            )
            .and_where(
                Expr::col(subscription_tags::Column::TagId)
                    .is_not_in(params.tags.iter().map(|e| e.id.to_string())),
            )
            .to_owned();

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        let mut query = Query::insert()
            .into_table(subscription_tags::Entity)
            .columns([
                subscription_tags::Column::SubscriptionId,
                subscription_tags::Column::TagId,
                subscription_tags::Column::UserId,
            ])
            .on_conflict(
                OnConflict::columns([
                    subscription_tags::Column::SubscriptionId,
                    subscription_tags::Column::TagId,
                ])
                .do_nothing()
                .to_owned(),
            )
            .to_owned();

        for tag in params.tags {
            query.values_panic([
                params.subscription_id.to_string().into(),
                tag.id.to_string().into(),
                tag.user_id.to_string().into(),
            ]);
        }

        tx.execute(self.db.get_database_backend().build(&query))
            .await?;

        Ok(())
    }

    async fn update_subscription_entry(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryUpdateParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if params.has_read {
            let query = Query::insert()
                .into_table(read_entries::Entity)
                .columns([
                    read_entries::Column::SubscriptionId,
                    read_entries::Column::FeedEntryId,
                    read_entries::Column::UserId,
                ])
                .values_panic([
                    params.subscription_id.to_string().into(),
                    params.feed_entry_id.to_string().into(),
                    params.user_id.to_string().into(),
                ])
                .on_conflict(
                    OnConflict::columns([
                        read_entries::Column::SubscriptionId,
                        read_entries::Column::FeedEntryId,
                    ])
                    .do_nothing()
                    .to_owned(),
                )
                .to_owned();

            tx.execute(self.db.get_database_backend().build(&query))
                .await?;
        } else {
            let query = Query::delete()
                .from_table(read_entries::Entity)
                .and_where(
                    Expr::col(read_entries::Column::SubscriptionId)
                        .eq(params.subscription_id.to_string()),
                )
                .and_where(
                    Expr::col(read_entries::Column::FeedEntryId)
                        .eq(params.feed_entry_id.to_string()),
                )
                .to_owned();

            tx.execute(self.db.get_database_backend().build(&query))
                .await?;
        }

        Ok(())
    }
}
