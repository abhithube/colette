use std::any::Any;

use chrono::{DateTime, Utc};
use colette_core::{
    common::{Transaction, TransactionManager},
    feed::ProcessedFeedEntry,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
};
use colette_model::{bookmarks, feed_entries, feeds, subscription_entries, subscriptions, tags};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseConnection,
    DatabaseTransaction, DbErr, EntityTrait, LinkDef, Linked, QueryFilter, RelationTrait,
    TransactionTrait,
    prelude::Expr,
    sea_query::{ExprTrait, OnConflict, SimpleExpr},
};
use url::Url;
use uuid::Uuid;

#[derive(Debug)]
pub struct SqliteTransaction {
    tx: DatabaseTransaction,
}

#[async_trait::async_trait]
impl Transaction for SqliteTransaction {
    async fn commit(self: Box<Self>) -> Result<(), DbErr> {
        self.tx.commit().await
    }

    async fn rollback(self: Box<Self>) -> Result<(), DbErr> {
        self.tx.rollback().await
    }

    fn as_any(&self) -> &dyn Any {
        &self.tx
    }
}

#[derive(Debug, Clone)]
pub struct SqliteTransactionManager {
    db: DatabaseConnection,
}

impl SqliteTransactionManager {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl TransactionManager for SqliteTransactionManager {
    async fn begin(&self) -> Result<Box<dyn Transaction>, DbErr> {
        let tx = self.db.begin().await?;

        Ok(Box::new(SqliteTransaction { tx }))
    }
}

pub(crate) async fn upsert_feed<C: ConnectionTrait>(
    conn: &C,
    link: Url,
    xml_url: Option<Url>,
    title: String,
    description: Option<String>,
    refreshed_at: Option<DateTime<Utc>>,
) -> Result<Uuid, DbErr> {
    let model = feeds::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4().into()),
        link: ActiveValue::Set(link.into()),
        xml_url: ActiveValue::Set(xml_url.map(Into::into)),
        title: ActiveValue::Set(title),
        description: ActiveValue::Set(description),
        refreshed_at: ActiveValue::Set(refreshed_at.map(|e| e.timestamp() as i32)),
    };

    let model = feeds::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([feeds::Column::Link])
                .update_columns([feeds::Column::XmlUrl])
                .update_columns([feeds::Column::Title])
                .update_columns([feeds::Column::Description])
                .update_columns([feeds::Column::RefreshedAt])
                .to_owned(),
        )
        .exec_with_returning(conn)
        .await?;

    Ok(model.id.parse().unwrap())
}

pub(crate) async fn upsert_entries<C: ConnectionTrait>(
    conn: &C,
    entries: Vec<ProcessedFeedEntry>,
    feed_id: Uuid,
) -> Result<(), DbErr> {
    let models = entries
        .into_iter()
        .map(|e| feed_entries::ActiveModel {
            link: ActiveValue::Set(e.link.into()),
            title: ActiveValue::Set(e.title),
            published_at: ActiveValue::Set(e.published.timestamp() as i32),
            description: ActiveValue::Set(e.description),
            thumbnail_url: ActiveValue::Set(e.thumbnail.map(Into::into)),
            author: ActiveValue::Set(e.author),
            feed_id: ActiveValue::Set(feed_id.into()),
            ..Default::default()
        })
        .collect::<Vec<_>>();

    feed_entries::Entity::insert_many(models)
        .do_nothing()
        .on_conflict(
            OnConflict::columns([feed_entries::Column::FeedId, feed_entries::Column::Link])
                .update_columns([
                    feed_entries::Column::Title,
                    feed_entries::Column::PublishedAt,
                    feed_entries::Column::Description,
                    feed_entries::Column::Author,
                    feed_entries::Column::ThumbnailUrl,
                ])
                .to_owned(),
        )
        .exec(conn)
        .await?;

    Ok(())
}

pub struct FeedEntryToSubscription;

impl Linked for FeedEntryToSubscription {
    type FromEntity = feed_entries::Entity;
    type ToEntity = subscriptions::Entity;

    fn link(&self) -> Vec<LinkDef> {
        vec![
            feeds::Relation::FeedEntries.def().rev(),
            feeds::Relation::Subscriptions.def(),
        ]
    }
}

pub(crate) async fn insert_many_subscription_entries(
    tx: &DatabaseTransaction,
    feed_id: Uuid,
) -> Result<(), DbErr> {
    let models = feed_entries::Entity::find()
        .find_also_linked(FeedEntryToSubscription)
        .filter(feed_entries::Column::FeedId.eq(feed_id.to_string()))
        .all(tx)
        .await
        .map(|e| {
            e.into_iter()
                .filter_map(|(fe, subscription)| {
                    subscription.map(|subscription| subscription_entries::ActiveModel {
                        id: ActiveValue::Set(Uuid::new_v4().to_string()),
                        feed_entry_id: ActiveValue::Set(fe.id),
                        subscription_id: ActiveValue::Set(subscription.id),
                        user_id: ActiveValue::Set(subscription.user_id),
                        ..Default::default()
                    })
                })
                .collect::<Vec<_>>()
        })?;

    subscription_entries::Entity::insert_many(models)
        .on_conflict(
            OnConflict::columns([
                subscription_entries::Column::SubscriptionId,
                subscription_entries::Column::FeedEntryId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .exec_without_returning(tx)
        .await?;

    Ok(())
}

pub(crate) async fn upsert_bookmark<C: ConnectionTrait>(
    conn: &C,
    link: Url,
    title: String,
    thumbnail_url: Option<Url>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    user_id: Uuid,
) -> Result<Uuid, DbErr> {
    let model = bookmarks::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4().into()),
        link: ActiveValue::Set(link.into()),
        title: ActiveValue::Set(title),
        thumbnail_url: ActiveValue::Set(thumbnail_url.map(Into::into)),
        published_at: ActiveValue::Set(published_at.map(|e| e.timestamp() as i32)),
        author: ActiveValue::Set(author),
        user_id: ActiveValue::Set(user_id.into()),
        ..Default::default()
    };

    let mut keys = bookmarks::Entity::insert(model)
        .on_conflict(
            OnConflict::columns([bookmarks::Column::UserId, bookmarks::Column::Link])
                .update_columns([
                    bookmarks::Column::Title,
                    bookmarks::Column::ThumbnailUrl,
                    bookmarks::Column::PublishedAt,
                    bookmarks::Column::Author,
                ])
                .to_owned(),
        )
        .exec_with_returning_keys(conn)
        .await?;

    Ok(keys.swap_remove(0).parse().unwrap())
}

pub(crate) async fn upsert_tag(
    tx: &DatabaseTransaction,
    title: String,
    user_id: Uuid,
) -> Result<Uuid, DbErr> {
    let model = tags::Entity::find()
        .filter(tags::Column::UserId.eq(user_id.to_string()))
        .filter(tags::Column::Title.eq(title.clone()))
        .one(tx)
        .await?;

    let tag_id = match model {
        Some(model) => model.id.parse().unwrap(),
        _ => {
            let id = Uuid::new_v4();
            let model = tags::ActiveModel {
                id: ActiveValue::Set(id.into()),
                title: ActiveValue::Set(title),
                user_id: ActiveValue::Set(user_id.into()),
                ..Default::default()
            };
            model.insert(tx).await?;

            id
        }
    };

    Ok(tag_id)
}

pub(crate) trait ToColumn {
    fn to_column(self) -> Expr;
}

pub(crate) trait ToSql {
    fn to_sql(self) -> SimpleExpr;
}

impl ToSql for (Expr, TextOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            TextOp::Equals(value) => column.to_owned().eq(value),
            TextOp::Contains(value) => column.to_owned().like(format!("%{}%", value)),
            TextOp::StartsWith(value) => column.to_owned().like(format!("{}%", value)),
            TextOp::EndsWith(value) => column.to_owned().like(format!("%{}", value)),
        }
    }
}

impl ToSql for (Expr, NumberOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            NumberOp::Equals(value) => column.to_owned().eq(value),
            NumberOp::LessThan(value) => column.to_owned().lt(value),
            NumberOp::GreaterThan(value) => column.to_owned().gt(value),
            NumberOp::Between(value) => column.to_owned().between(value.start, value.end),
        }
    }
}

impl ToSql for (Expr, BooleanOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            BooleanOp::Equals(value) => column.to_owned().eq(value),
        }
    }
}

impl ToSql for (Expr, DateOp) {
    fn to_sql(self) -> SimpleExpr {
        let (column, op) = self;

        match op {
            DateOp::Before(value) => column.to_owned().lt(value.timestamp()),
            DateOp::After(value) => column.to_owned().gt(value.timestamp()),
            DateOp::Between(value) => column
                .to_owned()
                .between(value.start.timestamp(), value.end.timestamp()),
            DateOp::InLast(value) => Expr::cust("strftime('%s', 'now')")
                .sub(column.to_owned())
                .lt(value),
        }
    }
}
