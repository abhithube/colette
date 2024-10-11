use chrono::{DateTime, Utc};
use sea_query::{
    ColumnDef, ColumnType, DeleteStatement, Expr, ForeignKey, ForeignKeyAction, Iden, Index,
    IndexCreateStatement, InsertStatement, OnConflict, Query, SelectStatement, Table,
    TableCreateStatement,
};

use crate::{common::WithTimestamps, feed::Feed, profile_feed_entry::ProfileFeedEntry};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum FeedEntry {
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(FeedEntry::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(FeedEntry::Id, ColumnType::Integer)
                .not_null()
                .primary_key()
                .auto_increment(),
        )
        .col(
            ColumnDef::new_with_type(FeedEntry::Link, ColumnType::Text)
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new_with_type(FeedEntry::Title, ColumnType::Text).not_null())
        .col(ColumnDef::new_with_type(FeedEntry::PublishedAt, timestamp_type.clone()).not_null())
        .col(ColumnDef::new_with_type(
            FeedEntry::Description,
            ColumnType::Text,
        ))
        .col(ColumnDef::new_with_type(
            FeedEntry::Author,
            ColumnType::Text,
        ))
        .col(ColumnDef::new_with_type(
            FeedEntry::ThumbnailUrl,
            ColumnType::Text,
        ))
        .col(ColumnDef::new_with_type(FeedEntry::FeedId, ColumnType::Integer).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(FeedEntry::Table, FeedEntry::FeedId)
                .to(Feed::Table, Feed::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_feed_id_link_index() -> IndexCreateStatement {
    Index::create()
        .name(format!(
            "{feed_entry}_{feed_id}_{link}_idx",
            feed_entry = FeedEntry::Table.to_string(),
            feed_id = FeedEntry::FeedId.to_string(),
            link = FeedEntry::Link.to_string()
        ))
        .table(FeedEntry::Table)
        .if_not_exists()
        .col(FeedEntry::FeedId)
        .col(FeedEntry::Link)
        .unique()
        .to_owned()
}

pub struct InsertMany {
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub fn select_many_by_feed_id(feed_id: i32) -> SelectStatement {
    Query::select()
        .column(FeedEntry::Id)
        .from(FeedEntry::Table)
        .and_where(Expr::col(FeedEntry::FeedId).eq(feed_id))
        .to_owned()
}

pub fn insert_many(data: Vec<InsertMany>, feed_id: i32) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(FeedEntry::Table)
        .columns([
            FeedEntry::Link,
            FeedEntry::Title,
            FeedEntry::PublishedAt,
            FeedEntry::Description,
            FeedEntry::Author,
            FeedEntry::ThumbnailUrl,
            FeedEntry::FeedId,
            FeedEntry::UpdatedAt,
        ])
        .on_conflict(
            OnConflict::columns([FeedEntry::FeedId, FeedEntry::Link])
                .update_columns([
                    FeedEntry::Title,
                    FeedEntry::PublishedAt,
                    FeedEntry::Description,
                    FeedEntry::Author,
                    FeedEntry::ThumbnailUrl,
                    FeedEntry::FeedId,
                    FeedEntry::UpdatedAt,
                ])
                .to_owned(),
        )
        .returning_col(FeedEntry::Id)
        .to_owned();

    for fe in data {
        query.values_panic([
            fe.link.into(),
            fe.title.into(),
            fe.published_at.into(),
            fe.description.into(),
            fe.author.into(),
            fe.thumbnail_url.into(),
            feed_id.into(),
            Expr::current_timestamp().into(),
        ]);
    }

    query
}

pub fn delete_many() -> DeleteStatement {
    let subquery = Query::select()
        .from(ProfileFeedEntry::Table)
        .and_where(
            Expr::col((ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId))
                .equals((FeedEntry::Table, FeedEntry::Id)),
        )
        .to_owned();

    Query::delete()
        .from_table(FeedEntry::Table)
        .and_where(Expr::exists(subquery).not())
        .to_owned()
}
