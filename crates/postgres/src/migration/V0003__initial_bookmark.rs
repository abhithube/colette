use colette_sql::bookmark::Bookmark;
use sea_query::{ColumnDef, ColumnType, PostgresQueryBuilder, Table};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [Table::create()
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
        .col(
            ColumnDef::new_with_type(Bookmark::PublishedAt, ColumnType::TimestampWithTimeZone)
                .not_null(),
        )
        .col(ColumnDef::new_with_type(Bookmark::Author, ColumnType::Text))
        .with_timestamps(ColumnType::TimestampWithTimeZone)
        .build(PostgresQueryBuilder)]
    .join("; ")
}
