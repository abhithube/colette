use sea_query::{
    ColumnDef, ColumnType, DeleteStatement, Expr, ForeignKey, ForeignKeyAction, Iden,
    InsertStatement, Query, Table, TableCreateStatement,
};
use uuid::Uuid;

use crate::{common::WithTimestamps, profile::Profile, smart_feed::SmartFeed};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
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

pub fn create_table(
    id_type: ColumnType,
    field_type: ColumnType,
    operation_type: ColumnType,
    timestamp_type: ColumnType,
) -> TableCreateStatement {
    Table::create()
        .table(SmartFeedFilter::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(SmartFeedFilter::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new_with_type(SmartFeedFilter::Field, field_type).not_null())
        .col(ColumnDef::new_with_type(SmartFeedFilter::Operation, operation_type).not_null())
        .col(ColumnDef::new_with_type(SmartFeedFilter::Value, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(SmartFeedFilter::SmartFeedId, id_type.clone()).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId)
                .to(SmartFeed::Table, SmartFeed::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .col(ColumnDef::new_with_type(SmartFeedFilter::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(SmartFeedFilter::Table, SmartFeedFilter::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
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
