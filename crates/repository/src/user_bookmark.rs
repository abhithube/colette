use std::fmt::Write;

use colette_core::bookmark::Cursor;
use sea_query::{
    Alias, CommonTableExpression, DeleteStatement, Expr, Iden, InsertStatement, JoinType,
    OnConflict, Query, SelectStatement, SimpleExpr, UpdateStatement, WithQuery,
};
use uuid::Uuid;

use crate::{bookmark::Bookmark, tag::Tag, user_bookmark_tag::UserBookmarkTag};

#[allow(dead_code)]
pub enum UserBookmark {
    Table,
    Id,
    UserId,
    BookmarkId,
    CollectionId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for UserBookmark {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "user_bookmarks",
                Self::Id => "id",
                Self::UserId => "user_id",
                Self::BookmarkId => "bookmark_id",
                Self::CollectionId => "collection_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub fn select(
    id: Option<Uuid>,
    collection_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
    jsonb_agg: SimpleExpr,
    tags_subquery: Option<SimpleExpr>,
) -> WithQuery {
    let pf_id = Alias::new("pf_id");

    let tags = Alias::new("tags");

    let json_tags_cte = Query::select()
        .expr_as(
            Expr::col((UserBookmark::Table, UserBookmark::Id)),
            pf_id.clone(),
        )
        .expr_as(jsonb_agg, tags.clone())
        .from(UserBookmark::Table)
        .join(
            JoinType::InnerJoin,
            UserBookmarkTag::Table,
            Expr::col((UserBookmarkTag::Table, UserBookmarkTag::UserBookmarkId))
                .eq(Expr::col((UserBookmark::Table, UserBookmark::Id))),
        )
        .join(
            JoinType::InnerJoin,
            Tag::Table,
            Expr::col((Tag::Table, Tag::Id))
                .eq(Expr::col((UserBookmarkTag::Table, UserBookmarkTag::TagId))),
        )
        .group_by_col((UserBookmark::Table, UserBookmark::Id))
        .to_owned();

    let json_tags = Alias::new("json_tags");

    let mut select = Query::select()
        .columns([
            (UserBookmark::Table, UserBookmark::Id),
            (UserBookmark::Table, UserBookmark::CreatedAt),
            (UserBookmark::Table, UserBookmark::CollectionId),
        ])
        .columns([
            (Bookmark::Table, Bookmark::Link),
            (Bookmark::Table, Bookmark::Title),
            (Bookmark::Table, Bookmark::ThumbnailUrl),
            (Bookmark::Table, Bookmark::PublishedAt),
            (Bookmark::Table, Bookmark::Author),
        ])
        .column((json_tags.clone(), tags))
        .from(UserBookmark::Table)
        .join(
            JoinType::InnerJoin,
            Bookmark::Table,
            Expr::col((Bookmark::Table, Bookmark::Id))
                .eq(Expr::col((UserBookmark::Table, UserBookmark::BookmarkId))),
        )
        .join(
            JoinType::LeftJoin,
            json_tags.clone(),
            Expr::col((json_tags.clone(), pf_id))
                .eq(Expr::col((UserBookmark::Table, UserBookmark::Id))),
        )
        .and_where(Expr::col((UserBookmark::Table, UserBookmark::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((UserBookmark::Table, UserBookmark::Id)).eq(e)))
        .and_where_option(
            collection_id
                .map(|e| Expr::col((UserBookmark::Table, UserBookmark::CollectionId)).eq(e)),
        )
        .and_where_option(tags_subquery)
        .and_where_option(cursor.map(|e| {
            Expr::col((UserBookmark::Table, UserBookmark::CreatedAt)).gt(Expr::val(e.created_at))
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

pub fn select_by_unique_index(user_id: Uuid, bookmark_id: i32) -> SelectStatement {
    Query::select()
        .column(UserBookmark::Id)
        .from(UserBookmark::Table)
        .and_where(Expr::col(UserBookmark::UserId).eq(user_id))
        .and_where(Expr::col(UserBookmark::BookmarkId).eq(bookmark_id))
        .to_owned()
}

pub fn insert(
    id: Option<Uuid>,
    bookmark_id: i32,
    user_id: Uuid,
    collection_id: Option<Uuid>,
) -> InsertStatement {
    let mut columns = vec![
        UserBookmark::BookmarkId,
        UserBookmark::UserId,
        UserBookmark::CollectionId,
    ];
    let mut values: Vec<SimpleExpr> =
        vec![bookmark_id.into(), user_id.into(), collection_id.into()];

    if let Some(id) = id {
        columns.push(UserBookmark::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(UserBookmark::Table)
        .columns(columns)
        .values_panic(values)
        .on_conflict(
            OnConflict::columns([UserBookmark::UserId, UserBookmark::BookmarkId])
                .do_nothing()
                .to_owned(),
        )
        .returning_col(UserBookmark::Id)
        .to_owned()
}

pub fn update(id: Uuid, user_id: Uuid, collection_id: Option<Option<Uuid>>) -> UpdateStatement {
    let mut query = Query::update()
        .table(UserBookmark::Table)
        .value(UserBookmark::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(UserBookmark::Id).eq(id))
        .and_where(Expr::col(UserBookmark::UserId).eq(user_id))
        .to_owned();

    if let Some(collection_id) = collection_id {
        query.value(UserBookmark::CollectionId, collection_id);
    }

    query
}

pub fn delete(id: Uuid, user_id: Uuid) -> DeleteStatement {
    Query::delete()
        .from_table(UserBookmark::Table)
        .and_where(Expr::col((UserBookmark::Table, UserBookmark::Id)).eq(id))
        .and_where(Expr::col((UserBookmark::Table, UserBookmark::UserId)).eq(user_id))
        .to_owned()
}
