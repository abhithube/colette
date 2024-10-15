use colette_sql::{
    common::{WithPk, WithTimestamps},
    feed::Feed,
    feed_entry::FeedEntry,
};
use sea_query::{
    ColumnDef, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

pub fn migration() -> String {
    [
        Table::create()
            .table(Feed::Table)
            .if_not_exists()
            .with_integer_pk()
            .col(ColumnDef::new(Feed::Link).text().not_null().unique_key())
            .col(ColumnDef::new(Feed::Title).text().not_null())
            .col(ColumnDef::new(Feed::Url).text())
            .with_timestamps()
            .build(PostgresQueryBuilder),
        Table::create()
            .table(FeedEntry::Table)
            .if_not_exists()
            .with_integer_pk()
            .col(
                ColumnDef::new(FeedEntry::Link)
                    .text()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(FeedEntry::Title).text().not_null())
            .col(
                ColumnDef::new(FeedEntry::PublishedAt)
                    .timestamp_with_time_zone()
                    .not_null(),
            )
            .col(ColumnDef::new(FeedEntry::Description).text())
            .col(ColumnDef::new(FeedEntry::Author).text())
            .col(ColumnDef::new(FeedEntry::ThumbnailUrl).text())
            .col(ColumnDef::new(FeedEntry::FeedId).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(FeedEntry::Table, FeedEntry::FeedId)
                    .to(Feed::Table, Feed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
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
