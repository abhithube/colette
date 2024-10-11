use colette_core::bookmark::Cursor;
use sea_query::{
    Alias, ColumnDef, ColumnType, CommonTableExpression, DeleteStatement, Expr, ForeignKey,
    ForeignKeyAction, Iden, Index, IndexCreateStatement, InsertStatement, JoinType, OnConflict,
    Query, SelectStatement, SimpleExpr, Table, TableCreateStatement, WithClause, WithQuery,
};
use uuid::Uuid;

use crate::{
    bookmark::Bookmark, common::WithTimestamps, profile::Profile,
    profile_bookmark_tag::ProfileBookmarkTag, tag::Tag,
};

#[derive(sea_query::Iden)]
pub enum ProfileBookmark {
    Table,
    Id,
    ProfileId,
    BookmarkId,
    CreatedAt,
    UpdatedAt,
}

pub fn create_table(id_type: ColumnType, timestamp_type: ColumnType) -> TableCreateStatement {
    Table::create()
        .table(ProfileBookmark::Table)
        .if_not_exists()
        .col(
            ColumnDef::new_with_type(ProfileBookmark::Id, id_type.clone())
                .not_null()
                .primary_key(),
        )
        .col(ColumnDef::new_with_type(ProfileBookmark::ProfileId, id_type).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileBookmark::Table, ProfileBookmark::ProfileId)
                .to(Profile::Table, Profile::Id)
                .on_delete(ForeignKeyAction::Cascade),
        )
        .col(ColumnDef::new_with_type(ProfileBookmark::BookmarkId, ColumnType::Integer).not_null())
        .foreign_key(
            ForeignKey::create()
                .from(ProfileBookmark::Table, ProfileBookmark::BookmarkId)
                .to(Bookmark::Table, Bookmark::Id)
                .on_delete(ForeignKeyAction::Restrict),
        )
        .with_timestamps(timestamp_type)
        .to_owned()
}

pub fn create_profile_id_bookmark_id_index() -> IndexCreateStatement {
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
        .to_owned()
}

pub fn select(
    id: Option<Uuid>,
    profile_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    jsonb_agg: SimpleExpr,
    tags_subquery: Option<SimpleExpr>,
) -> WithQuery {
    let pf_id = Alias::new("pf_id");

    let tags = Alias::new("tags");

    let json_tags_cte = Query::select()
        .expr_as(
            Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)),
            pf_id.clone(),
        )
        .expr_as(jsonb_agg, tags.clone())
        .from(ProfileBookmark::Table)
        .join(
            JoinType::InnerJoin,
            ProfileBookmarkTag::Table,
            Expr::col((
                ProfileBookmarkTag::Table,
                ProfileBookmarkTag::ProfileBookmarkId,
            ))
            .eq(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id)).eq(Expr::col((
                ProfileBookmarkTag::Table,
                ProfileBookmarkTag::TagId,
            ))),
        )
        .group_by_col((ProfileBookmark::Table, ProfileBookmark::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");

    let mut select = Query::select()
        .columns([
            (ProfileBookmark::Table, ProfileBookmark::Id),
            (ProfileBookmark::Table, ProfileBookmark::CreatedAt),
        ])
        .columns([
            (Bookmark::Table, Bookmark::Link),
            (Bookmark::Table, Bookmark::Title),
            (Bookmark::Table, Bookmark::ThumbnailUrl),
            (Bookmark::Table, Bookmark::PublishedAt),
            (Bookmark::Table, Bookmark::Author),
        ])
        .column((json_tags.clone(), tags.clone()))
        .from(ProfileBookmark::Table)
        .join(
            JoinType::Join,
            Bookmark::Table,
            Expr::col((Bookmark::Table, Bookmark::Id)).eq(Expr::col((
                ProfileBookmark::Table,
                ProfileBookmark::BookmarkId,
            ))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id.clone()))
                .eq(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id))),
        )
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .and_where_option(
            id.map(|e| Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(e)),
        )
        .and_where_option(tags_subquery)
        .and_where_option(cursor.map(|e| {
            Expr::col((ProfileBookmark::Table, ProfileBookmark::CreatedAt))
                .gt(Expr::val(e.created_at))
        }))
        .to_owned();

    if let Some(limit) = limit {
        select.limit(limit);
    }

    select.with(
        WithClause::new()
            .cte(
                CommonTableExpression::new()
                    .query(json_tags_cte)
                    .table_name(json_tags)
                    .to_owned(),
            )
            .to_owned(),
    )
}

pub fn select_by_unique_index(profile_id: Uuid, bookmark_id: i32) -> SelectStatement {
    Query::select()
        .column(ProfileBookmark::Id)
        .from(ProfileBookmark::Table)
        .and_where(Expr::col(ProfileBookmark::ProfileId).eq(profile_id))
        .and_where(Expr::col(ProfileBookmark::BookmarkId).eq(bookmark_id))
        .to_owned()
}

pub fn insert(id: Uuid, bookmark_id: i32, profile_id: Uuid) -> InsertStatement {
    Query::insert()
        .into_table(ProfileBookmark::Table)
        .columns([
            ProfileBookmark::Id,
            ProfileBookmark::BookmarkId,
            ProfileBookmark::ProfileId,
        ])
        .values_panic([id.into(), bookmark_id.into(), profile_id.into()])
        .on_conflict(
            OnConflict::columns([ProfileBookmark::ProfileId, ProfileBookmark::BookmarkId])
                .do_nothing()
                .to_owned(),
        )
        .to_owned()
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileBookmark::Table)
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(id))
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .to_owned()
}
