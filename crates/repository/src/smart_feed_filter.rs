use std::fmt::Write;

use sea_query::{DeleteStatement, Expr, Iden, InsertStatement, Query, SimpleExpr};
use uuid::Uuid;

pub enum SmartFeedFilter {
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

impl Iden for SmartFeedFilter {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "smart_feed_filters",
                Self::Id => "id",
                Self::Field => "field",
                Self::Operation => "operation",
                Self::Value => "value",
                Self::SmartFeedId => "smart_feed_id",
                Self::ProfileId => "profile_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

#[derive(Debug, Clone)]
pub enum Field {
    Type,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    HasRead,
}

impl Iden for Field {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Type => "field",
                Self::Link => "link",
                Self::Title => "title",
                Self::PublishedAt => "published_at",
                Self::Description => "description",
                Self::Author => "author",
                Self::HasRead => "has_read",
            }
        )
        .unwrap();
    }
}

#[derive(Debug, Clone)]
pub enum Operation {
    Type,
    Eq,
    Ne,
    Like,
    NotLike,
    GreaterThan,
    LessThan,
    InLastXSec,
}

impl Iden for Operation {
    fn unquoted(&self, s: &mut dyn std::fmt::Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Type => "operation",
                Self::Eq => "=",
                Self::Ne => "!=",
                Self::Like => "LIKE",
                Self::NotLike => "NOT LIKE",
                Self::GreaterThan => ">",
                Self::LessThan => "<",
                Self::InLastXSec => "in_last_x_sec",
            }
        )
        .unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct InsertMany {
    pub id: Option<Uuid>,
    pub field: Field,
    pub operation: Operation,
    pub value: String,
}

pub fn insert_many(data: &[InsertMany], smart_feed_id: Uuid, profile_id: Uuid) -> InsertStatement {
    let mut columns = vec![
        SmartFeedFilter::Field,
        SmartFeedFilter::Operation,
        SmartFeedFilter::Value,
        SmartFeedFilter::SmartFeedId,
        SmartFeedFilter::ProfileId,
    ];
    if data.iter().any(|e| e.id.is_some()) {
        columns.push(SmartFeedFilter::Id);
    }

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

    for sff in data {
        let mut values: Vec<SimpleExpr> = vec![
            sff.field.to_string().into(),
            sff.operation.to_string().into(),
            (*sff.value).into(),
            smart_feed_id.into(),
            profile_id.into(),
        ];
        if let Some(id) = sff.id {
            values.push(id.into());
        }

        query.values_panic(values);
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
