use colette_sql::{
    bookmark::Bookmark,
    common::{WithPk, WithTimestamps},
};
use sea_query::{ColumnDef, PostgresQueryBuilder, Table};

pub fn migration() -> String {
    [Table::create()
        .table(Bookmark::Table)
        .if_not_exists()
        .with_integer_pk()
        .col(
            ColumnDef::new(Bookmark::Link)
                .text()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(Bookmark::Title).text().not_null())
        .col(ColumnDef::new(Bookmark::ThumbnailUrl).text())
        .col(
            ColumnDef::new(Bookmark::PublishedAt)
                .timestamp_with_time_zone()
                .not_null(),
        )
        .col(ColumnDef::new(Bookmark::Author).text())
        .with_timestamps()
        .build(PostgresQueryBuilder)]
    .join("; ")
}
