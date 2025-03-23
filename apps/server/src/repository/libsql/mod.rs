pub use api_key::LibsqlApiKeyRepository;
pub use backup::LibsqlBackupRepository;
pub use bookmark::LibsqlBookmarkRepository;
pub use collection::LibsqlCollectionRepository;
pub use feed::LibsqlFeedRepository;
pub use feed_entry::LibsqlFeedEntryRepository;
pub use job::LibsqlJobRepository;
use libsql::Value;
use sea_query::{
    DeleteStatement, InsertStatement, QueryBuilder, SelectStatement, UpdateStatement, WithQuery,
};
pub use stream::LibsqlStreamRepository;
pub use subscription::LibsqlSubscriptionRepository;
pub use subscription_entry::LibsqlSubscriptionEntryRepository;
pub use tag::LibsqlTagRepository;

mod api_key;
mod backup;
mod bookmark;
mod collection;
mod feed;
mod feed_entry;
mod job;
mod stream;
mod subscription;
mod subscription_entry;
mod tag;

#[derive(Debug)]
pub struct LibsqlValue(pub sea_query::Value);

#[derive(Debug)]
pub struct LibsqlValues(pub Vec<LibsqlValue>);

impl LibsqlValues {
    pub fn into_params(self) -> Vec<Value> {
        self.0.into_iter().map(Value::from).collect()
    }
}

pub trait LibsqlBinder {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues);
}

impl From<LibsqlValue> for Value {
    fn from(value: LibsqlValue) -> Value {
        match value.0 {
            sea_query::Value::Bool(v) => v.into(),
            sea_query::Value::Int(v) => v.into(),
            sea_query::Value::BigInt(v) => v.into(),
            // sea_query::Value::BigUnsigned(v) => v.into_value(),
            sea_query::Value::Float(v) => v.into(),
            sea_query::Value::Double(v) => v.into(),
            sea_query::Value::String(v) => match v {
                Some(v) => (*v).into(),
                None => Value::Null,
            },
            sea_query::Value::Char(v) => match v {
                Some(v) => v.to_string().into(),
                None => Value::Null,
            },
            sea_query::Value::Bytes(v) => match v {
                Some(v) => (*v).into(),
                None => Value::Null,
            },
            sea_query::Value::ChronoDateTimeUtc(v) => match v {
                Some(v) => (*v).format("%F %T%.f%:z").to_string().into(),
                None => Value::Null,
            },
            sea_query::Value::Uuid(v) => match v {
                Some(v) => (*v).to_string().into(),
                None => Value::Null,
            },
            _ => unimplemented!(),
        }
    }
}

impl LibsqlBinder for SelectStatement {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues) {
        let (query, values) = self.build(query_builder);

        (
            query,
            LibsqlValues(values.into_iter().map(LibsqlValue).collect()),
        )
    }
}

impl LibsqlBinder for InsertStatement {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues) {
        let (query, values) = self.build(query_builder);

        (
            query,
            LibsqlValues(values.into_iter().map(LibsqlValue).collect()),
        )
    }
}

impl LibsqlBinder for UpdateStatement {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues) {
        let (query, values) = self.build(query_builder);

        (
            query,
            LibsqlValues(values.into_iter().map(LibsqlValue).collect()),
        )
    }
}

impl LibsqlBinder for DeleteStatement {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues) {
        let (query, values) = self.build(query_builder);

        (
            query,
            LibsqlValues(values.into_iter().map(LibsqlValue).collect()),
        )
    }
}

impl LibsqlBinder for WithQuery {
    fn build_libsql<T: QueryBuilder>(&self, query_builder: T) -> (String, LibsqlValues) {
        let (query, values) = self.build(query_builder);

        (
            query,
            LibsqlValues(values.into_iter().map(LibsqlValue).collect()),
        )
    }
}
