use colette_core::{
    Feed,
    common::Transaction,
    feed::{
        ConflictError, Error, FeedById, FeedCreateData, FeedFindParams, FeedRepository,
        FeedScrapedData, FeedUpdateData,
    },
};
use colette_model::{
    FeedWithTagsAndCount, feeds, subscription_entries, subscription_tags, subscriptions, tags,
};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction,
    DbErr, EntityTrait, IntoSimpleExpr, LoaderTrait, QueryFilter, QueryOrder, QuerySelect,
    QueryTrait, TransactionTrait,
    prelude::Expr,
    sea_query::{Func, Query},
};
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

        let unread_counts = subscription_entries::Entity::find()
            .select_only()
            .expr(Func::count(Expr::col(subscription_entries::Column::Id)))
            .filter(subscription_entries::Column::HasRead.eq(false))
            .filter(
                subscription_entries::Column::SubscriptionId
                    .is_in(subscription_models.iter().map(|e| e.id.as_str())),
            )
            .group_by(subscription_entries::Column::SubscriptionId)
            .into_tuple::<i64>()
            .all(&self.db)
            .await?;

        let feeds = subscription_models
            .into_iter()
            .zip(feed_models.into_iter())
            .zip(unread_counts.into_iter())
            .zip(tag_models.into_iter())
            .map(|(((subscription, feed), unread_count), tags)| {
                FeedWithTagsAndCount {
                    subscription,
                    feed,
                    tags,
                    unread_count,
                }
                .into()
            })
            .collect();

        Ok(feeds)
    }

    async fn find_feed_by_id(&self, tx: &dyn Transaction, id: Uuid) -> Result<FeedById, Error> {
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

        Ok(FeedById {
            id: id.parse().unwrap(),
            user_id: user_id.parse().unwrap(),
        })
    }

    async fn create_feed(&self, data: FeedCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let Some(feed_model) = feeds::Entity::find()
            .filter(
                Condition::any()
                    .add(feeds::Column::Link.eq(data.url.to_string()))
                    .add(feeds::Column::XmlUrl.eq(data.url.to_string())),
            )
            .one(&tx)
            .await?
        else {
            return Err(Error::Conflict(ConflictError::NotCached(data.url)));
        };

        let id = Uuid::new_v4();
        let subscription_model = subscriptions::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title),
            user_id: ActiveValue::Set(data.user_id.into()),
            feed_id: ActiveValue::Set(feed_model.id),
            ..Default::default()
        };
        subscription_model.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(ConflictError::AlreadyExists(data.url)),
            _ => Error::Database(e),
        })?;

        common::insert_many_subscription_entries(&tx, feed_model.id).await?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }

    async fn update_feed(
        &self,
        tx: &dyn Transaction,
        id: Uuid,
        data: FeedUpdateData,
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

    async fn delete_feed(&self, tx: &dyn Transaction, id: Uuid) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        subscriptions::Entity::delete_by_id(id.to_string())
            .exec(tx)
            .await?;

        Ok(())
    }

    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error> {
        if data.link_to_users {
            let tx = self.db.begin().await?;

            let feed_id = common::upsert_feed(&tx, data.feed.link, Some(data.url)).await?;
            common::upsert_entries(&tx, data.feed.entries, feed_id).await?;

            common::insert_many_subscription_entries(&tx, feed_id).await?;

            tx.commit().await?;
        } else {
            let tx = self.db.begin().await?;

            let feed_id = common::upsert_feed(&tx, data.feed.link, Some(data.url)).await?;
            common::upsert_entries(&tx, data.feed.entries, feed_id).await?;

            tx.commit().await?;
        }

        Ok(())
    }

    async fn stream_urls(&self) -> Result<BoxStream<Result<String, Error>>, Error> {
        let urls = feeds::Entity::find()
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
            .map_err(Error::Database)
            .boxed();

        Ok(urls)
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
