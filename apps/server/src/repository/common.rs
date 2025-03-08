use std::any::Any;

use chrono::{DateTime, Utc};
use colette_core::{
    common::{Transaction, TransactionManager},
    feed::ProcessedFeedEntry,
    filter::{BooleanOp, DateOp, NumberOp, TextOp},
};
use colette_model::{bookmarks, feed_entries, feeds, tags};
use sea_orm::{
    ConnectionTrait, DatabaseConnection, DatabaseTransaction, DbErr, TransactionTrait,
    sea_query::{Expr, ExprTrait, OnConflict, Query, SimpleExpr},
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
    let query = Query::insert()
        .into_table(feeds::Entity)
        .columns([
            feeds::Column::Id,
            feeds::Column::Link,
            feeds::Column::XmlUrl,
            feeds::Column::Title,
            feeds::Column::Description,
            feeds::Column::RefreshedAt,
        ])
        .values_panic([
            Uuid::new_v4().to_string().into(),
            link.to_string().into(),
            xml_url.map(String::from).into(),
            title.into(),
            description.into(),
            refreshed_at.map(|e| e.timestamp()).into(),
        ])
        .on_conflict(
            OnConflict::column(feeds::Column::Link)
                .update_columns([
                    feeds::Column::XmlUrl,
                    feeds::Column::Title,
                    feeds::Column::Description,
                    feeds::Column::RefreshedAt,
                ])
                .to_owned(),
        )
        .returning_col(feeds::Column::Id)
        .to_owned();

    let result = conn
        .query_one(conn.get_database_backend().build(&query))
        .await?
        .unwrap();

    Ok(result
        .try_get_by_index::<String>(0)
        .unwrap()
        .parse()
        .unwrap())
}

pub(crate) async fn upsert_entries<C: ConnectionTrait>(
    conn: &C,
    entries: Vec<ProcessedFeedEntry>,
    feed_id: Uuid,
) -> Result<(), DbErr> {
    let mut query = Query::insert()
        .into_table(feed_entries::Entity)
        .columns([
            feed_entries::Column::Id,
            feed_entries::Column::Link,
            feed_entries::Column::Title,
            feed_entries::Column::PublishedAt,
            feed_entries::Column::Description,
            feed_entries::Column::Author,
            feed_entries::Column::ThumbnailUrl,
            feed_entries::Column::FeedId,
        ])
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
        .to_owned();

    let feed_id = feed_id.to_string();
    for entry in entries {
        query.values_panic([
            Uuid::new_v4().to_string().into(),
            entry.link.to_string().into(),
            entry.title.into(),
            entry.published.timestamp().into(),
            entry.description.into(),
            entry.author.into(),
            entry.thumbnail.map(String::from).into(),
            feed_id.clone().into(),
        ]);
    }

    conn.execute(conn.get_database_backend().build(&query))
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
    let query = Query::insert()
        .into_table(bookmarks::Entity)
        .columns([
            bookmarks::Column::Id,
            bookmarks::Column::Link,
            bookmarks::Column::Title,
            bookmarks::Column::ThumbnailUrl,
            bookmarks::Column::PublishedAt,
            bookmarks::Column::Author,
            bookmarks::Column::UserId,
        ])
        .values_panic([
            Uuid::new_v4().to_string().into(),
            link.to_string().into(),
            title.into(),
            thumbnail_url.map(String::from).into(),
            published_at.map(|e| e.timestamp()).into(),
            author.into(),
            user_id.to_string().into(),
        ])
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
        .returning_col(bookmarks::Column::Id)
        .to_owned();

    let result = conn
        .query_one(conn.get_database_backend().build(&query))
        .await?
        .unwrap();

    Ok(result
        .try_get_by_index::<String>(0)
        .unwrap()
        .parse()
        .unwrap())
}

pub(crate) async fn upsert_tag(
    tx: &DatabaseTransaction,
    title: String,
    user_id: Uuid,
) -> Result<Uuid, DbErr> {
    let query = Query::select()
        .column(tags::Column::Id)
        .from(tags::Entity)
        .and_where(Expr::col(tags::Column::UserId).eq(user_id.to_string()))
        .and_where(Expr::col(tags::Column::UserId).eq(title.clone()))
        .to_owned();

    let result = tx
        .query_one(tx.get_database_backend().build(&query))
        .await?;

    let tag_id = match result {
        Some(model) => model
            .try_get_by_index::<String>(0)
            .unwrap()
            .parse()
            .unwrap(),
        _ => {
            let id = Uuid::new_v4();

            let query = Query::insert()
                .into_table(tags::Entity)
                .columns([tags::Column::Id, tags::Column::Title, tags::Column::UserId])
                .values_panic([
                    id.to_string().into(),
                    title.into(),
                    user_id.to_string().into(),
                ])
                .to_owned();

            tx.execute(tx.get_database_backend().build(&query)).await?;

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
