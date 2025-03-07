use colette_core::{
    Subscription,
    common::Transaction,
    subscription::{
        Error, SubscriptionById, SubscriptionCreateData, SubscriptionEntryUpdateData,
        SubscriptionEntryUpdateParams, SubscriptionFindParams, SubscriptionRepository,
        SubscriptionUpdateData,
    },
};
use colette_model::{
    SubscriptionWithTagsAndCount, feed_entries, feeds, read_entries, subscription_tags,
    subscriptions, tags,
};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, IntoSimpleExpr, LoaderTrait, QueryFilter, QueryOrder, QuerySelect, QueryTrait,
    TransactionTrait,
    prelude::Expr,
    sea_query::{Func, Query},
};
use uuid::Uuid;

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
        let models = subscriptions::Entity::find()
            .find_also_related(feeds::Entity)
            .apply_if(params.id, |query, id| {
                query.filter(subscriptions::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.user_id, |query, user_id| {
                query.filter(subscriptions::Column::UserId.eq(user_id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        subscriptions::Column::Title.into_simple_expr(),
                        subscriptions::Column::Id.into_simple_expr(),
                    ])
                    .gt(Expr::tuple([
                        Expr::value(cursor.title),
                        Expr::value(cursor.id.to_string()),
                    ])),
                )
            })
            .apply_if(params.tags, |query, tags| {
                query.filter(Expr::exists(
                    Query::select()
                        .expr(Expr::val(1))
                        .from(subscription_tags::Entity)
                        .and_where(
                            Expr::col(subscription_tags::Column::SubscriptionId)
                                .eq(Expr::col(subscriptions::Column::Id)),
                        )
                        .and_where(
                            subscription_tags::Column::TagId
                                .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                        )
                        .to_owned(),
                ))
            })
            .order_by_asc(subscriptions::Column::Title)
            .order_by_asc(subscriptions::Column::Id)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await?;

        let (subscription_models, feed_models): (Vec<_>, Vec<_>) = models
            .into_iter()
            .filter_map(|(subscription, feed)| feed.map(|f| (subscription, f)))
            .unzip();

        let tag_models = subscription_models
            .load_many_to_many(
                tags::Entity::find().order_by_asc(tags::Column::Title),
                subscription_tags::Entity,
                &self.db,
            )
            .await?;

        let unread_counts = feed_entries::Entity::find()
            .select_only()
            .expr(Func::count(Expr::col((
                feed_entries::Entity,
                feed_entries::Column::Id,
            ))))
            .inner_join(subscriptions::Entity)
            .filter(
                subscriptions::Column::Id.is_in(subscription_models.iter().map(|e| e.id.as_str())),
            )
            .filter(
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
            .group_by(subscriptions::Column::Id)
            .into_tuple::<i64>()
            .all(&self.db)
            .await?;

        let subscriptions = subscription_models
            .into_iter()
            .zip(feed_models.into_iter())
            .zip(unread_counts.into_iter())
            .zip(tag_models.into_iter())
            .map(|(((subscription, feed), unread_count), tags)| {
                SubscriptionWithTagsAndCount {
                    subscription,
                    feed,
                    tags,
                    unread_count,
                }
                .into()
            })
            .collect();

        Ok(subscriptions)
    }

    async fn find_subscription_by_id(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
    ) -> Result<SubscriptionById, Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let Some((id, user_id)) = subscriptions::Entity::find()
            .select_only()
            .columns([subscriptions::Column::Id, subscriptions::Column::UserId])
            .filter(subscriptions::Column::Id.eq(id.to_string()))
            .into_tuple::<(String, String)>()
            .one(tx)
            .await?
        else {
            return Err(Error::NotFound(id));
        };

        Ok(SubscriptionById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn create_subscription(&self, data: SubscriptionCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let id = Uuid::new_v4();
        let model = subscriptions::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title),
            user_id: ActiveValue::Set(data.user_id.into()),
            feed_id: ActiveValue::Set(data.feed_id.into()),
            ..Default::default()
        };
        model.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(data.feed_id),
            _ => Error::Database(e),
        })?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }

    async fn update_subscription(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: SubscriptionUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let mut model = subscriptions::ActiveModel {
            id: ActiveValue::Set(id.to_string()),
            ..Default::default()
        };

        if let Some(title) = data.title {
            model.title = ActiveValue::Set(title);
        }

        if model.is_changed() {
            model.update(tx).await?;
        }

        // if let Some(tags) = data.tags {
        //     link_tags(&tx, tags, id, params.user_id).await?;
        // }

        // tx.commit().await?;

        Ok(())
    }

    async fn delete_subscription(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        subscriptions::Entity::delete_by_id(id.to_string())
            .exec(tx)
            .await?;

        Ok(())
    }

    async fn update_subscription_entry(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionEntryUpdateParams,
        data: SubscriptionEntryUpdateData,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        if data.has_read {
            let model = read_entries::ActiveModel {
                feed_entry_id: ActiveValue::Set(params.feed_entry_id.into()),
                subscription_id: ActiveValue::Set(params.subscription_id.into()),
                user_id: ActiveValue::Set(params.user_id.into()),
                ..Default::default()
            };

            read_entries::Entity::insert(model)
                .on_conflict_do_nothing()
                .exec(tx)
                .await?;
        } else {
            read_entries::Entity::delete_by_id((
                params.subscription_id.to_string(),
                params.feed_entry_id.to_string(),
            ))
            .exec(tx)
            .await?;
        }

        Ok(())
    }
}

async fn link_tags(
    tx: &DatabaseTransaction,
    tags: Vec<Uuid>,
    subscription_id: Uuid,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let subscription_id = subscription_id.to_string();
    let user_id = user_id.to_string();
    let tag_ids = tags.iter().map(|e| e.to_string());

    subscription_tags::Entity::delete_many()
        .filter(subscription_tags::Column::TagId.is_not_in(tag_ids.clone()))
        .exec(tx)
        .await?;

    let models = tag_ids.map(|e| subscription_tags::ActiveModel {
        subscription_id: ActiveValue::Set(subscription_id.clone()),
        tag_id: ActiveValue::Set(e),
        user_id: ActiveValue::Set(user_id.clone()),
        ..Default::default()
    });
    subscription_tags::Entity::insert_many(models)
        .on_conflict_do_nothing()
        .exec(tx)
        .await?;

    Ok(())
}
