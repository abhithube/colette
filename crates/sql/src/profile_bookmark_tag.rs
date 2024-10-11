use sea_query::{
    ColumnDef, ColumnType, DeleteStatement, Expr, ForeignKey, ForeignKeyAction, Index,
    InsertStatement, OnConflict, Query, Table, TableCreateStatement,
};
use uuid::Uuid;

use crate::{
    common::WithTimestamps,
    profile::Profile,
    profile_bookmark::ProfileBookmark,
    tag::{build_titles_subquery, Tag},
};

#[derive(sea_query::Iden)]
pub enum ProfileBookmarkTag {
    Table,
    ProfileBookmarkId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(ProfileBookmarkTag::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(ProfileBookmarkTag::ProfileBookmarkId, id_type.clone())
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
        .col(ColumnDef::new_with_type(ProfileBookmarkTag::TagId, id_type.clone()).not_null())
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
        .col(ColumnDef::new_with_type(ProfileBookmarkTag::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileBookmarkTag::Table, ProfileBookmarkTag::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub struct InsertMany {
    pub profile_bookmark_id: Uuid,
    pub tag_id: Uuid,
}

pub fn insert_many(data: Vec<InsertMany>, profile_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(ProfileBookmarkTag::Table)
        .columns([
            ProfileBookmarkTag::ProfileBookmarkId,
            ProfileBookmarkTag::TagId,
            ProfileBookmarkTag::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([
                ProfileBookmarkTag::ProfileBookmarkId,
                ProfileBookmarkTag::TagId,
            ])
            .do_nothing()
            .to_owned(),
        )
        .to_owned();

    for pbt in data {
        query.values_panic([
            pbt.profile_bookmark_id.into(),
            pbt.tag_id.into(),
            profile_id.into(),
        ]);
    }

    query
}

pub fn delete_many_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileBookmarkTag::Table)
        .and_where(
            Expr::col(ProfileBookmarkTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id)),
        )
        .to_owned()
}

pub fn delete_many_not_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileBookmarkTag::Table)
        .and_where(Expr::col(ProfileBookmarkTag::ProfileId).eq(profile_id))
        .and_where(
            Expr::col(ProfileBookmarkTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id))
                .not(),
        )
        .to_owned()
}
