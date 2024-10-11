use colette_sql::{
    profile::Profile, profile_bookmark::ProfileBookmark, profile_bookmark_tag::ProfileBookmarkTag,
    profile_feed::ProfileFeed, profile_feed_tag::ProfileFeedTag, tag::Tag,
};
use sea_query::{
    ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(Tag::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(Tag::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new_with_type(Tag::Title, ColumnType::Text).not_null())
            .col(ColumnDef::new_with_type(Tag::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(Tag::Table, Tag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        Index::create()
            .name(format!(
                "{tag}_{profile_id}_{title}_idx",
                tag = Tag::Table.to_string(),
                profile_id = Tag::ProfileId.to_string(),
                title = Tag::Title.to_string()
            ))
            .table(Tag::Table)
            .if_not_exists()
            .col(Tag::ProfileId)
            .col(Tag::Title)
            .unique()
            .build(PostgresQueryBuilder),
        Table::create()
            .table(ProfileFeedTag::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(ProfileFeedTag::ProfileFeedId, ColumnType::Uuid)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)
                    .to(ProfileFeed::Table, ProfileFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new_with_type(ProfileFeedTag::TagId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedTag::Table, ProfileFeedTag::TagId)
                    .to(Tag::Table, Tag::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .primary_key(
                Index::create()
                    .col(ProfileFeedTag::ProfileFeedId)
                    .col(ProfileFeedTag::TagId),
            )
            .col(ColumnDef::new_with_type(ProfileFeedTag::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        Table::create()
            .table(ProfileBookmarkTag::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(ProfileBookmarkTag::ProfileBookmarkId, ColumnType::Uuid)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(
                        ProfileBookmarkTag::Table,
                        ProfileBookmarkTag::ProfileBookmarkId,
                    )
                    .to(ProfileBookmark::Table, ProfileBookmark::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new_with_type(ProfileBookmarkTag::TagId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::TagId)
                    .to(Tag::Table, Tag::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .primary_key(
                Index::create()
                    .col(ProfileBookmarkTag::ProfileBookmarkId)
                    .col(ProfileBookmarkTag::TagId),
            )
            .col(
                ColumnDef::new_with_type(ProfileBookmarkTag::ProfileId, ColumnType::Uuid)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
