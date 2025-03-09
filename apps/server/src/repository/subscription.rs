use std::collections::HashMap;

use colette_core::{
    Subscription, Tag,
    common::Transaction,
    subscription::{
        Error, SubscriptionById, SubscriptionCreateParams, SubscriptionDeleteParams,
        SubscriptionEntryUpdateParams, SubscriptionFindByIdParams, SubscriptionFindParams,
        SubscriptionRepository, SubscriptionTagsLinkParams, SubscriptionUpdateParams,
    },
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect, IntoUpdate,
    feed_entry::UnreadCountSelectMany,
    subscription_tag::{
        SubscriptionTagDeleteMany, SubscriptionTagSelectMany, SubscriptionTagUpsertMany,
    },
};
use sea_orm::{ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, FromQueryResult};

use super::{common::parse_timestamp, feed::FeedRow};

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
        let subscription_rows = SubscriptionRow::find_by_statement(
            self.db.get_database_backend().build(&params.into_select()),
        )
        .all(&self.db)
        .await?;

        let subscription_ids = subscription_rows.iter().map(|e| e.id.to_string());

        let tag_select = SubscriptionTagSelectMany {
            subscription_ids: subscription_ids.clone(),
        };

        let tag_rows = SubscriptionTagRow::find_by_statement(
            self.db
                .get_database_backend()
                .build(&tag_select.into_select()),
        )
        .all(&self.db)
        .await?;

        let unread_count_select = UnreadCountSelectMany { subscription_ids };

        let unread_count_results = self
            .db
            .query_all(
                self.db
                    .get_database_backend()
                    .build(&unread_count_select.into_select()),
            )
            .await?;

        let mut tag_row_map = HashMap::<String, Vec<SubscriptionTagRow>>::new();
        let mut unread_count_map = HashMap::<String, i64>::new();

        for row in tag_rows {
            tag_row_map
                .entry(row.subscription_id.clone())
                .or_default()
                .push(row);
        }

        for row in unread_count_results {
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

        let id = params.id;

        let Some(result) = tx
            .query_one(self.db.get_database_backend().build(&params.into_select()))
            .await?
        else {
            return Err(Error::NotFound(id));
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

        let feed_id = params.feed_id;

        tx.execute(self.db.get_database_backend().build(&params.into_insert()))
            .await
            .map_err(|e| match e {
                DbErr::RecordNotInserted => Error::Conflict(feed_id),
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

        tx.execute(self.db.get_database_backend().build(&params.into_update()))
            .await?;

        Ok(())
    }

    async fn delete_subscription(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionDeleteParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        tx.execute(self.db.get_database_backend().build(&params.into_delete()))
            .await?;

        Ok(())
    }

    async fn link_tags(
        &self,
        tx: &dyn Transaction,
        params: SubscriptionTagsLinkParams,
    ) -> Result<(), Error> {
        let tx = tx.as_any().downcast_ref::<DatabaseTransaction>().unwrap();

        let delete_many = SubscriptionTagDeleteMany {
            subscription_id: params.subscription_id,
            tag_ids: params.tags.iter().map(|e| e.id.to_string()),
        };

        tx.execute(
            self.db
                .get_database_backend()
                .build(&delete_many.into_delete()),
        )
        .await?;

        let upsert_many = SubscriptionTagUpsertMany {
            subscription_id: params.subscription_id,
            tags: params.tags,
        };

        tx.execute(
            self.db
                .get_database_backend()
                .build(&upsert_many.into_insert()),
        )
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
            tx.execute(self.db.get_database_backend().build(&params.into_insert()))
                .await?;
        } else {
            tx.execute(self.db.get_database_backend().build(&params.into_delete()))
                .await?;
        }

        Ok(())
    }
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionRow {
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,

    pub feed_id: String,
    pub link: String,
    pub xml_url: Option<String>,
    pub feed_title: String,
    pub description: Option<String>,
    pub refreshed_at: Option<i32>,
}

#[derive(sea_orm::FromQueryResult)]
pub struct SubscriptionTagRow {
    pub subscription_id: String,
    pub id: String,
    pub title: String,
    pub user_id: String,
    pub created_at: i32,
    pub updated_at: i32,
}

impl From<SubscriptionTagRow> for Tag {
    fn from(value: SubscriptionTagRow) -> Self {
        Self {
            id: value.id.parse().unwrap(),
            title: value.title,
            user_id: value.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.created_at),
            updated_at: parse_timestamp(value.updated_at),
            ..Default::default()
        }
    }
}

pub struct SubscriptionWithTagsAndCount {
    pub subscription: SubscriptionRow,
    pub tags: Option<Vec<SubscriptionTagRow>>,
    pub unread_count: Option<i64>,
}

impl From<SubscriptionWithTagsAndCount> for Subscription {
    fn from(value: SubscriptionWithTagsAndCount) -> Self {
        Self {
            id: value.subscription.id.parse().unwrap(),
            title: value.subscription.title,
            user_id: value.subscription.user_id.parse().unwrap(),
            created_at: parse_timestamp(value.subscription.created_at),
            updated_at: parse_timestamp(value.subscription.updated_at),
            feed: FeedRow {
                id: value.subscription.feed_id,
                link: value.subscription.link,
                xml_url: value.subscription.xml_url,
                title: value.subscription.feed_title,
                description: value.subscription.description,
                refreshed_at: value.subscription.refreshed_at,
            }
            .into(),
            tags: value.tags.map(|e| e.into_iter().map(Into::into).collect()),
            unread_count: value.unread_count,
        }
    }
}
