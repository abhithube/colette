use sea_query::{
    ColumnDef, ColumnType, DeleteStatement, Expr, ForeignKey, ForeignKeyAction, Index,
    InsertStatement, OnConflict, Query, Table, TableCreateStatement,
};
use uuid::Uuid;

use crate::{
    common::WithTimestamps,
    profile::Profile,
    profile_feed::ProfileFeed,
    tag::{build_titles_subquery, Tag},
};

#[derive(sea_query::Iden)]
pub enum ProfileFeedTag {
    Table,
    ProfileFeedId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(ProfileFeedTag::Table)
        .if_not_exists()
        .col(ColumnDef::new_with_type(ProfileFeedTag::ProfileFeedId, id_type.clone()).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileFeedId)
                .to(ProfileFeed::Table, ProfileFeed::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .col(ColumnDef::new_with_type(ProfileFeedTag::TagId, id_type.clone()).not_null())
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
        .col(ColumnDef::new_with_type(ProfileFeedTag::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileFeedTag::Table, ProfileFeedTag::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub struct InsertMany {
    pub profile_feed_id: Uuid,
    pub tag_id: Uuid,
}

pub fn insert_many(data: Vec<InsertMany>, profile_id: Uuid) -> InsertStatement {
    let mut query = Query::insert()
        .into_table(ProfileFeedTag::Table)
        .columns([
            ProfileFeedTag::ProfileFeedId,
            ProfileFeedTag::TagId,
            ProfileFeedTag::ProfileId,
        ])
        .on_conflict(
            OnConflict::columns([ProfileFeedTag::ProfileFeedId, ProfileFeedTag::TagId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned();

    for pft in data {
        query.values_panic([
            pft.profile_feed_id.into(),
            pft.tag_id.into(),
            profile_id.into(),
        ]);
    }

    query
}

pub fn delete_many_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(
            Expr::col(ProfileFeedTag::TagId).in_subquery(build_titles_subquery(titles, profile_id)),
        )
        .to_owned()
}

pub fn delete_many_not_in_titles(titles: &[String], profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileFeedTag::Table)
        .and_where(Expr::col(ProfileFeedTag::ProfileId).eq(profile_id))
        .and_where(
            Expr::col(ProfileFeedTag::TagId)
                .in_subquery(build_titles_subquery(titles, profile_id))
                .not(),
        )
        .to_owned()
}
