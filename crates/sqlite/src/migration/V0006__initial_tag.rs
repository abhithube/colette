use colette_sql::{
    common::{WithPk, WithTimestamps},
    profile::Profile,
    profile_bookmark::ProfileBookmark,
    profile_bookmark_tag::ProfileBookmarkTag,
    profile_feed::ProfileFeed,
    profile_feed_tag::ProfileFeedTag,
    tag::Tag,
};
use sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Iden, Index, SqliteQueryBuilder, Table};

pub fn migration() -> String {
    [
        Table::create()
            .table(Tag::Table)
            .if_not_exists()
            .with_uuid_pk()
            .col(ColumnDef::new(Tag::Title).text().not_null())
            .col(ColumnDef::new(Tag::ProfileId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(Tag::Table, Tag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
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
            .build(SqliteQueryBuilder),
        Table::create()
            .table(ProfileFeedTag::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ProfileFeedTag::ProfileFeedId)
                    .uuid()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)
                    .to(ProfileFeed::Table, ProfileFeed::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(ColumnDef::new(ProfileFeedTag::TagId).uuid().not_null())
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
            .col(ColumnDef::new(ProfileFeedTag::ProfileId).uuid().not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
        Table::create()
            .table(ProfileBookmarkTag::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(ProfileBookmarkTag::ProfileBookmarkId)
                    .uuid()
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
            .col(ColumnDef::new(ProfileBookmarkTag::TagId).uuid().not_null())
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
                ColumnDef::new(ProfileBookmarkTag::ProfileId)
                    .uuid()
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .with_timestamps()
            .build(SqliteQueryBuilder),
    ]
    .join("; ")
}
