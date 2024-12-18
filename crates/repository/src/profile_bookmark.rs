use std::fmt::Write;

use colette_core::bookmark::Cursor;
use sea_query::{
    Alias, CommonTableExpression, DeleteStatement, Expr, Iden, InsertStatement, JoinType,
    OnConflict, Query, SelectStatement, SimpleExpr, WithQuery,
};
use uuid::Uuid;

use crate::{bookmark::Bookmark, profile_bookmark_tag::ProfileBookmarkTag, tag::Tag};

#[allow(dead_code)]
pub enum ProfileBookmark {
    Table,
    Id,
    ProfileId,
    BookmarkId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for ProfileBookmark {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "profile_bookmarks",
                Self::Id => "id",
                Self::ProfileId => "profile_id",
                Self::BookmarkId => "bookmark_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
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
            JoinType::InnerJoin,
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
        Query::with()
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

pub fn insert(id: Option<Uuid>, bookmark_id: i32, profile_id: Uuid) -> InsertStatement {
    let mut columns = vec![ProfileBookmark::BookmarkId, ProfileBookmark::ProfileId];
    let mut values: Vec<SimpleExpr> = vec![bookmark_id.into(), profile_id.into()];

    if let Some(id) = id {
        columns.push(ProfileBookmark::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(ProfileBookmark::Table)
        .columns(columns)
        .values_panic(values)
        .on_conflict(
            OnConflict::columns([ProfileBookmark::ProfileId, ProfileBookmark::BookmarkId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(ProfileBookmark::Id)
        .to_owned()
}

pub fn delete(id: Uuid, profile_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(ProfileBookmark::Table)
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::Id)).eq(id))
        .and_where(Expr::col((ProfileBookmark::Table, ProfileBookmark::ProfileId)).eq(profile_id))
        .to_owned()
}
