use chrono::{NaiveDate, NaiveTime};
use sea_query::{
    DeleteStatement, InsertStatement, QueryBuilder, SelectStatement, UpdateStatement, Value,
    WithQuery,
};
use worker::{wasm_bindgen::JsValue, D1Argument};

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
