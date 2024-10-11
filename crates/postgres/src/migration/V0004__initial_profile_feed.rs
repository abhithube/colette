use colette_sql::{
    feed::Feed, feed_entry::FeedEntry, profile::Profile, profile_feed::ProfileFeed,
    profile_feed_entry::ProfileFeedEntry,
};
use sea_query::{
    ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(ProfileFeed::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(ProfileFeed::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new_with_type(
                ProfileFeed::Title,
                ColumnType::Text,
            ))
            .col(
                ColumnDef::new_with_type(ProfileFeed::Pinned, ColumnType::Boolean)
                    .not_null()
                    .default(false),
            )
            .col(ColumnDef::new_with_type(ProfileFeed::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeed::Table, ProfileFeed::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new_with_type(ProfileFeed::FeedId, ColumnType::Integer).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeed::Table, ProfileFeed::FeedId)
                    .to(Feed::Table, Feed::Id)
                    .on_delete(ForeignKeyAction::Restrict),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
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
            .col(
                ColumnDef::new_with_type(ProfileFeedEntry::Id, ColumnType::Integer)
                    .not_null()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new_with_type(ProfileFeedEntry::HasRead, ColumnType::Boolean)
                    .not_null()
                    .default(false),
            )
            .col(
                ColumnDef::new_with_type(ProfileFeedEntry::ProfileFeedId, ColumnType::Uuid)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileFeedId)
                    .to(ProfileFeed::Table, ProfileFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(
                ColumnDef::new_with_type(ProfileFeedEntry::FeedEntryId, ColumnType::Integer)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::FeedEntryId)
                    .to(FeedEntry::Table, FeedEntry::Id)
                    .on_delete(ForeignKeyAction::Restrict),
            )
            .col(ColumnDef::new_with_type(ProfileFeedEntry::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedEntry::Table, ProfileFeedEntry::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
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
