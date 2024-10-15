use colette_sql::{
    common::{WithPk, WithTimestamps},
    profile::Profile,
    smart_feed::SmartFeed,
    smart_feed_filter::SmartFeedFilter,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Iden, Index, SqliteQueryBuilder, Table};

pub fn migration() -> String {
    [
        Table::create()
            .table(SmartFeed::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(SmartFeed::Title).text().not_null())
            .col(ColumnDef::new(SmartFeed::ProfileId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeed::Table, SmartFeed::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
        Index::create()
            .name(format!(
                "{smart_feed}_{profile_id}_{title}_idx",
                smart_feed = SmartFeed::Table.to_string(),
                profile_id = SmartFeed::ProfileId.to_string(),
                title = SmartFeed::Title.to_string()
            ))
            .table(SmartFeed::Table)
            .if_not_exists()
            .col(SmartFeed::ProfileId)
            .col(SmartFeed::Title)
            .unique()
            .build(SqliteQueryBuilder),
        Table::create()
            .table(SmartFeedFilter::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(SmartFeedFilter::Field).text().not_null())
            .col(ColumnDef::new(SmartFeedFilter::Operation).text().not_null())
            .col(ColumnDef::new(SmartFeedFilter::Value).text().not_null())
            .col(
                ColumnDef::new(SmartFeedFilter::SmartFeedId)
                    .uuid()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeedFilter::Table, SmartFeedFilter::SmartFeedId)
                    .to(SmartFeed::Table, SmartFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(SmartFeedFilter::ProfileId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(SmartFeedFilter::Table, SmartFeedFilter::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
    ]
    .join("; ")
}
