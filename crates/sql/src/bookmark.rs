use chrono::{DateTime, Utc};
use sea_query::{
    ColumnDef, ColumnType, Expr, InsertStatement, OnConflict, Query, Table, TableCreateStatement,
};

use crate::common::WithTimestamps;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum Bookmark {
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(Bookmark::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(Bookmark::Id, ColumnType::Integer)
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(
            ColumnDef::new_with_type(Bookmark::Link, ColumnType::Text)
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new_with_type(Bookmark::Title, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(
            Bookmark::ThumbnailUrl,
            ColumnType::Text,
        ))
        .col(ColumnDef::new_with_type(Bookmark::PublishedAt, timestamp_type.clone()).not_null())
        .col(ColumnDef::new_with_type(Bookmark::Author, ColumnType::Text))
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn insert(
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
) -> InsertStatement {
    Query::insert()
        .into_table(Bookmark::Table)
        .columns([
            Bookmark::Link,
            Bookmark::Title,
            Bookmark::ThumbnailUrl,
            Bookmark::PublishedAt,
            Bookmark::Author,
            Bookmark::UpdatedAt,
        ])
        .values_panic([
            link.into(),
            title.into(),
            thumbnail_url.into(),
            published_at.into(),
            author.into(),
            Expr::current_timestamp().into(),
        ])
        .on_conflict(
            OnConflict::column(Bookmark::Link)
                .update_columns([
                    Bookmark::Title,
                    Bookmark::ThumbnailUrl,
                    Bookmark::PublishedAt,
                    Bookmark::Author,
                    Bookmark::UpdatedAt,
                ])
                .to_owned(),
        )
        .returning_col(Bookmark::Id)
        .to_owned()
}
