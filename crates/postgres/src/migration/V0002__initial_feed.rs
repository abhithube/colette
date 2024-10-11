use colette_sql::{feed::Feed, feed_entry::FeedEntry};
use sea_query::{
    ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(Feed::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(Feed::Id, ColumnType::Integer)
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new_with_type(Feed::Link, ColumnType::Text)
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new_with_type(Feed::Title, ColumnType::Text).not_null())
            .col(ColumnDef::new_with_type(Feed::Url, ColumnType::Text))
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
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
            .col(
                ColumnDef::new_with_type(FeedEntry::PublishedAt, ColumnType::TimestampWithTimeZone)
                    .not_null(),
            )
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
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
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
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
