use colette_sql::{
    common::{WithPk, WithTimestamps},
    feed::Feed,
    feed_entry::FeedEntry,
    profile::Profile,
    profile_feed::ProfileFeed,
    profile_feed_entry::ProfileFeedEntry,
};
use sea_query::{
    ColumnDef, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

pub fn migration() -> String {
    [
        Table::create()
            .table(ProfileFeed::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(ProfileFeed::Title).text())
            .col(
                ColumnDef::new(ProfileFeed::Pinned)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new(ProfileFeed::ProfileId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeed::Table, ProfileFeed::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(ProfileFeed::FeedId).integer().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeed::Table, ProfileFeed::FeedId)
                    .to(Feed::Table, Feed::Id)
                    .on_delete(ForeignKeyAction::Restrict),
            )
            .with_timestamps()
            .build(PostgresQueryBuilder),
        Index::create()
            .name(format!(
                "{profile_feed}_{profile_id}_{feed_id}_idx",
                profile_feed = ProfileFeed::Table.to_string(),
                profile_id = ProfileFeed::ProfileId.to_string(),
                feed_id = ProfileFeed::FeedId.to_string()
            ))
            .table(ProfileFeed::Table)
            .if_not_exists()
            .col(ProfileFeed::ProfileId)
            .col(ProfileFeed::FeedId)
            .unique()
            .build(PostgresQueryBuilder),
        Table::create()
            .table(ProfileFeedEntry::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(
                ColumnDef::new(ProfileFeedEntry::HasRead)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new(ProfileFeedEntry::ProfileFeedId)
                    .uuid()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)
                    .to(ProfileFeed::Table, ProfileFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(
                ColumnDef::new(ProfileFeedEntry::FeedEntryId)
                    .integer()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId)
                    .to(FeedEntry::Table, FeedEntry::Id)
                    .on_delete(ForeignKeyAction::Restrict),
            )
            .col(
                ColumnDef::new(ProfileFeedEntry::ProfileId)
                    .uuid()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(PostgresQueryBuilder),
        Index::create()
            .name(format!(
                "{profile_feed_entry}_{profile_feed_id}_{feed_entry_id}_idx",
                profile_feed_entry = ProfileFeedEntry::Table.to_string(),
                profile_feed_id = ProfileFeedEntry::ProfileFeedId.to_string(),
                feed_entry_id = ProfileFeedEntry::FeedEntryId.to_string()
            ))
            .table(ProfileFeedEntry::Table)
            .if_not_exists()
            .col(ProfileFeedEntry::ProfileFeedId)
            .col(ProfileFeedEntry::FeedEntryId)
            .unique()
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
