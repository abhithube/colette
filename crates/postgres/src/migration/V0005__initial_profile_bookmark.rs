use colette_sql::{bookmark::Bookmark, profile::Profile, profile_bookmark::ProfileBookmark};
use sea_query::{
    ColumnDef, ColumnType, ForeignKey, ForeignKeyAction, Iden, Index, PostgresQueryBuilder, Table,
};

use crate::migration::common::WithTimestamps;

pub fn migration() -> String {
    [
        Table::create()
            .table(ProfileBookmark::Table)
            .if_not_exists()
            .col(
                ColumnDef::new_with_type(ProfileBookmark::Id, ColumnType::Uuid)
                    .not_null()
                    .primary_key(),
            )
            .col(ColumnDef::new_with_type(ProfileBookmark::ProfileId, ColumnType::Uuid).not_null())
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileBookmark::Table, ProfileBookmark::ProfileId)
                    .to(Profile::Table, Profile::Id)
                    .on_delete(ForeignKeyAction::Cascade),
            )
            .col(
                ColumnDef::new_with_type(ProfileBookmark::BookmarkId, ColumnType::Integer)
                    .not_null(),
            )
            .foreign_key(
                ForeignKey::create()
                    .from(ProfileBookmark::Table, ProfileBookmark::BookmarkId)
                    .to(Bookmark::Table, Bookmark::Id)
                    .on_delete(ForeignKeyAction::Restrict),
            )
            .with_timestamps(ColumnType::TimestampWithTimeZone)
            .build(PostgresQueryBuilder),
        Index::create()
            .name(format!(
                "{profile_bookmark}_{profile_id}_{bookmark_id}_idx",
                profile_bookmark = ProfileBookmark::Table.to_string(),
                profile_id = ProfileBookmark::ProfileId.to_string(),
                bookmark_id = ProfileBookmark::BookmarkId.to_string()
            ))
            .table(ProfileBookmark::Table)
            .if_not_exists()
            .col(ProfileBookmark::ProfileId)
            .col(ProfileBookmark::BookmarkId)
            .unique()
            .build(PostgresQueryBuilder),
    ]
    .join("; ")
}
