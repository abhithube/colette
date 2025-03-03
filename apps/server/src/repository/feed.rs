use colette_core::{
    Feed,
    common::IdParams,
    feed::{
        ConflictError, Error, FeedCreateData, FeedFindParams, FeedRepository, FeedScrapedData,
        FeedUpdateData,
    },
};
use colette_model::{
    FeedWithTagsAndCount, feeds, tags, user_feed_entries, user_feed_tags, user_feeds,
};
use futures::{StreamExt, TryStreamExt, stream::BoxStream};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction,
    DbErr, EntityTrait, IntoActiveModel, IntoSimpleExpr, LoaderTrait, ModelTrait, QueryFilter,
    QueryOrder, QuerySelect, QueryTrait, TransactionTrait,
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
        let models = user_feeds::Entity::find()
            .find_also_related(feeds::Entity)
            .filter(user_feeds::Column::UserId.eq(params.user_id.to_string()))
            .apply_if(params.id, |query, id| {
                query.filter(user_feeds::Column::Id.eq(id.to_string()))
            })
            .apply_if(params.cursor, |query, cursor| {
                query.filter(
                    Expr::tuple([
                        user_feeds::Column::Title.into_simple_expr(),
                        user_feeds::Column::Id.into_simple_expr(),
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
                        .from(user_feed_tags::Entity)
                        .and_where(
                            Expr::col(user_feed_tags::Column::UserFeedId)
                                .eq(Expr::col(user_feeds::Column::Id)),
                        )
                        .and_where(
                            user_feed_tags::Column::TagId
                                .is_in(tags.into_iter().map(String::from).collect::<Vec<_>>()),
                        )
                        .to_owned(),
                ))
            })
            .order_by_asc(user_feeds::Column::Title)
            .order_by_asc(user_feeds::Column::Id)
            .limit(params.limit.map(|e| e as u64))
            .all(&self.db)
            .await?;

        let (user_feed_models, feed_models): (Vec<_>, Vec<_>) = models
            .into_iter()
            .filter_map(|(uf, f)| f.map(|f| (uf, f)))
            .unzip();

        let tag_models = user_feed_models
            .load_many_to_many(
                tags::Entity::find().order_by_asc(tags::Column::Title),
                user_feed_tags::Entity,
                &self.db,
            )
            .await?;

        let unread_counts = user_feed_entries::Entity::find()
            .select_only()
            .expr(Func::count(Expr::col(user_feed_entries::Column::Id)))
            .filter(user_feed_entries::Column::HasRead.eq(false))
            .filter(
                user_feed_entries::Column::UserFeedId
                    .is_in(user_feed_models.iter().map(|e| e.id.as_str())),
            )
            .group_by(user_feed_entries::Column::UserFeedId)
            .into_tuple::<i64>()
            .all(&self.db)
            .await?;

        let feeds = user_feed_models
            .into_iter()
            .zip(feed_models.into_iter())
            .zip(unread_counts.into_iter())
            .zip(tag_models.into_iter())
            .map(|(((user_feed, feed), unread_count), tags)| {
                FeedWithTagsAndCount {
                    user_feed,
                    feed,
                    tags,
                    unread_count,
                }
                .into()
            })
            .collect();

        Ok(feeds)
    }

    async fn create_feed(&self, data: FeedCreateData) -> Result<Uuid, Error> {
        let tx = self.db.begin().await?;

        let Some(feed) = feeds::Entity::find()
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
        let user_feed = user_feeds::ActiveModel {
            id: ActiveValue::Set(id.into()),
            title: ActiveValue::Set(data.title),
            user_id: ActiveValue::Set(data.user_id.into()),
            feed_id: ActiveValue::Set(feed.id),
            ..Default::default()
        };
        user_feed.insert(&tx).await.map_err(|e| match e {
            DbErr::RecordNotInserted => Error::Conflict(ConflictError::AlreadyExists(data.url)),
            _ => Error::Database(e),
        })?;

        common::insert_many_user_feed_entries(&tx, feed.id).await?;

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, id, data.user_id).await?;
        }

        tx.commit().await?;

        Ok(id)
    }

    async fn update_feed(&self, params: IdParams, data: FeedUpdateData) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(feed) = user_feeds::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if feed.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        let mut feed = feed.into_active_model();

        if let Some(title) = data.title {
            feed.title = ActiveValue::Set(title);
        }

        if feed.is_changed() {
            feed.update(&tx).await?;
        }

        if let Some(tags) = data.tags {
            link_tags(&tx, tags, params.id, params.user_id).await?;
        }

        tx.commit().await?;

        Ok(())
    }

    async fn delete_feed(&self, params: IdParams) -> Result<(), Error> {
        let tx = self.db.begin().await?;

        let Some(user_feed) = user_feeds::Entity::find_by_id(params.id).one(&tx).await? else {
            return Err(Error::NotFound(params.id));
        };
        if user_feed.user_id != params.user_id.to_string() {
            return Err(Error::NotFound(params.id));
        }

        user_feed.delete(&tx).await?;

        tx.commit().await?;

        Ok(())
    }

    async fn save_scraped(&self, data: FeedScrapedData) -> Result<(), Error> {
        if data.link_to_users {
            let tx = self.db.begin().await?;

            let feed_id = common::upsert_feed(&tx, data.feed.link, Some(data.url)).await?;
            common::upsert_entries(&tx, data.feed.entries, feed_id).await?;

            common::insert_many_user_feed_entries(&tx, feed_id).await?;

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
            .inner_join(user_feeds::Entity)
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
    user_feed_id: Uuid,
    user_id: Uuid,
) -> Result<(), DbErr> {
    let user_feed_id = user_feed_id.to_string();
    let user_id = user_id.to_string();
    let tag_ids = tags.iter().map(|e| e.to_string());

    user_feed_tags::Entity::delete_many()
        .filter(user_feed_tags::Column::TagId.is_not_in(tag_ids.clone()))
        .exec(tx)
        .await?;

    let models = tag_ids.map(|e| user_feed_tags::ActiveModel {
        user_feed_id: ActiveValue::Set(user_feed_id.clone()),
        tag_id: ActiveValue::Set(e),
        user_id: ActiveValue::Set(user_id.clone()),
        ..Default::default()
    });
    user_feed_tags::Entity::insert_many(models)
        .on_conflict_do_nothing()
        .exec(tx)
        .await?;

    Ok(())
}
