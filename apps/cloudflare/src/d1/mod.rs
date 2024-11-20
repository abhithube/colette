use chrono::{NaiveDate, NaiveTime};
use sea_query::{
    DeleteStatement, InsertStatement, QueryBuilder, SelectStatement, UpdateStatement, Value,
    WithQuery,
};
use serde::Deserialize;
use worker::{wasm_bindgen::JsValue, D1Argument, D1Database, D1Result};

pub mod profile;
pub mod user;

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
            Value::Bool(v) => JsValue::from(*v),
            Value::TinyInt(v) => (*v).into(),
            Value::SmallInt(v) => (*v).into(),
            Value::Int(v) => (*v).into(),
            Value::BigInt(v) => (*v).into(),
            Value::TinyUnsigned(v) => (*v).into(),
            Value::SmallUnsigned(v) => (*v).into(),
            Value::Unsigned(v) => (*v).into(),
            Value::BigUnsigned(v) => (*v).into(),
            Value::Float(v) => (*v).into(),
            Value::Double(v) => (*v).into(),
            Value::String(v) => v.as_deref().into(),
            Value::Char(v) => v.as_ref().map(|e| e.to_string()).into(),
            Value::Bytes(v) => v.to_owned().map(|e| e.into_boxed_slice()).into(),
            Value::ChronoDate(v) => v
                .as_ref()
                .map(|e| {
                    e.and_time(NaiveTime::default())
                        .and_utc()
                        .timestamp_millis()
                })
                .into(),
            Value::ChronoTime(v) => v
                .as_ref()
                .map(|e| {
                    NaiveDate::default()
                        .and_time(**e)
                        .and_utc()
                        .timestamp_millis()
                })
                .into(),
            Value::ChronoDateTime(v) => v.as_ref().map(|e| e.and_utc().timestamp_millis()).into(),
            Value::ChronoDateTimeUtc(v) => v.as_ref().map(|e| e.timestamp_millis()).into(),
            Value::ChronoDateTimeLocal(v) => v.as_ref().map(|e| e.timestamp_millis()).into(),
            Value::ChronoDateTimeWithTimeZone(v) => v.as_ref().map(|e| e.timestamp_millis()).into(),
            Value::Uuid(v) => v.as_ref().map(|e| e.to_string()).into(),
            Value::Json(v) => v.as_ref().map(|e| e.to_string()).into(),
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
