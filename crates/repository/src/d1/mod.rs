pub use backup::D1BackupRepository;
pub use bookmark::D1BookmarkRepository;
pub use collection::D1CollectionRepository;
pub use feed::D1FeedRepository;
pub use feed_entry::D1FeedEntryRepository;
pub use folder::D1FolderRepository;
pub use scraper::D1ScraperRepository;
use sea_query::{
    DeleteStatement, InsertStatement, QueryBuilder, SelectStatement, UpdateStatement, Value,
    WithQuery,
};
use serde::Deserialize;
pub use smart_feed::D1SmartFeedRepository;
pub use tag::D1TagRepository;
pub use user::D1UserRepository;
use worker::{console_error, wasm_bindgen::JsValue, D1Argument, D1Database, D1Result};

mod backup;
mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod folder;
mod scraper;
mod smart_feed;
mod tag;
mod user;

#[derive(Clone, Debug, PartialEq)]
pub struct D1Value(pub sea_query::Value);
#[derive(Clone, Debug, PartialEq)]
pub struct D1Values(pub Vec<D1Value>);

pub trait D1Binder {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values);
}

impl D1Binder for SelectStatement {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values) {
        let (query, values) = self.build(query_builder);
        (query, D1Values(values.into_iter().map(D1Value).collect()))
    }
}

impl D1Binder for InsertStatement {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values) {
        let (query, values) = self.build(query_builder);
        (query, D1Values(values.into_iter().map(D1Value).collect()))
    }
}

impl D1Binder for UpdateStatement {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values) {
        let (query, values) = self.build(query_builder);
        (query, D1Values(values.into_iter().map(D1Value).collect()))
    }
}

impl D1Binder for DeleteStatement {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values) {
        let (query, values) = self.build(query_builder);
        (query, D1Values(values.into_iter().map(D1Value).collect()))
    }
}

impl D1Binder for WithQuery {
    fn build_d1<T: QueryBuilder>(&self, query_builder: T) -> (String, D1Values) {
        let (query, values) = self.build(query_builder);
        (query, D1Values(values.into_iter().map(D1Value).collect()))
    }
}

impl D1Argument for D1Value {
    fn js_value(&self) -> impl AsRef<JsValue> {
        match &self.0 {
            Value::Bool(v) => match v {
                Some(v) => JsValue::from_bool(*v),
                None => JsValue::null(),
            },
            Value::Int(v) => match v {
                Some(v) => JsValue::from_f64(*v as f64),
                None => JsValue::null(),
            },
            Value::BigInt(v) => match v {
                Some(v) => JsValue::from_f64(*v as f64),
                None => JsValue::null(),
            },
            Value::BigUnsigned(v) => match v {
                Some(v) => JsValue::from_f64(*v as f64),
                None => JsValue::null(),
            },
            Value::String(v) => match v {
                Some(v) => JsValue::from_str(v),
                None => JsValue::null(),
            },
            Value::ChronoDateTimeUtc(v) => match v {
                Some(v) => JsValue::from_str(&v.to_string()),
                None => JsValue::null(),
            },
            Value::Uuid(v) => match v {
                Some(v) => JsValue::from_str(&v.to_string()),
                None => JsValue::null(),
            },
            v => {
                console_error!("{:?}", v);
                unimplemented!()
            }
        }
    }
}

#[worker::send]
pub(crate) async fn run(
    db: &D1Database,
    sql: String,
    values: D1Values,
) -> worker::Result<D1Result> {
    db.prepare(sql).bind_refs(&values.0).unwrap().run().await
}

#[worker::send]
pub(crate) async fn all(
    db: &D1Database,
    sql: String,
    values: D1Values,
) -> worker::Result<D1Result> {
    db.prepare(sql).bind_refs(&values.0).unwrap().all().await
}

#[worker::send]
pub(crate) async fn first<T>(
    db: &D1Database,
    sql: String,
    values: D1Values,
    col_name: Option<&str>,
) -> worker::Result<Option<T>>
where
    T: for<'a> Deserialize<'a>,
{
    db.prepare(sql)
        .bind_refs(&values.0)
        .unwrap()
        .first(col_name)
        .await
}

#[worker::send]
pub(crate) async fn batch(
    db: &D1Database,
    queries: Vec<(String, D1Values)>,
) -> worker::Result<Vec<D1Result>> {
    let stmts = queries
        .into_iter()
        .map(|(sql, values)| db.prepare(sql).bind_refs(&values.0).unwrap())
        .collect::<Vec<_>>();

    db.batch(stmts).await
}

#[derive(Debug, thiserror::Error)]
pub enum D1Error {
    #[error("unique constraint failed")]
    UniqueConstraint,
    #[error("{0}")]
    Other(String),
    #[error(transparent)]
    Unknown(worker::Error),
}

impl From<worker::Error> for D1Error {
    fn from(value: worker::Error) -> Self {
        match value {
            worker::Error::D1(d1) => match d1.cause() {
                e if e.starts_with("UNIQUE constraint failed") => Self::UniqueConstraint,
                e => Self::Other(e),
            },
            _ => Self::Unknown(value),
        }
    }
}
