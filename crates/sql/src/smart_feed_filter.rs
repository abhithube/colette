use std::fmt;

use sea_query::{DeleteStatement, Expr, InsertStatement, Query};
use sqlx::types::Uuid;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum SmartFeedFilter {
    Table,
    Id,
    Field,
    Operation,
    Value,
    SmartFeedId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone)]
pub struct InsertMany {
    pub id: Uuid,
    pub field: Field,
    pub operation: Operation,
    pub value: String,
}

pub fn insert_many(
    data: Vec<InsertMany>,
    smart_feed_id: Uuid,
    profile_id: Uuid,
) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(SmartFeedFilter::Table)
        .columns([
            SmartFeedFilter::Id,
            SmartFeedFilter::Field,
            SmartFeedFilter::Operation,
            SmartFeedFilter::Value,
            SmartFeedFilter::SmartFeedId,
            SmartFeedFilter::ProfileId,
        ])
        .to_owned();

    for t in data {
        query.values_panic([
            t.id.into(),
            t.field.to_string().into(),
            t.operation.to_string().into(),
            t.value.into(),
            smart_feed_id.into(),
            profile_id.into(),
        ]);
    }

    query
}

pub fn delete_many(profile_id: Uuid, smart_feed_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(SmartFeedFilter::Table)
        .and_where(Expr::col(SmartFeedFilter::ProfileId).eq(profile_id))
        .and_where(Expr::col(SmartFeedFilter::SmartFeedId).eq(smart_feed_id))
        .to_owned()
}

#[derive(Debug, Clone)]
pub enum Field {
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    HasRead,
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Link => "link",
            Self::Title => "title",
            Self::PublishedAt => "published_at",
            Self::Description => "description",
            Self::Author => "author",
            Self::HasRead => "has_read",
        };

        write!(f, "{}", str)
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Eq,
    Ne,
    Like,
    NotLike,
    GreaterThan,
    LessThan,
    InLastXSec,
}

impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Like => "LIKE",
            Self::NotLike => "NOT LIKE",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::InLastXSec => "in_last_x_sec",
        };

        write!(f, "{}", str)
    }
}
