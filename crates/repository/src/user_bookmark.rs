use std::fmt::Write;

use chrono::{DateTime, Utc};
use colette_core::bookmark::Cursor;
use sea_query::{
    Alias, CommonTableExpression, DeleteStatement, Expr, Iden, InsertStatement, JoinType, Query,
    SelectStatement, SimpleExpr, UpdateStatement, WithQuery,
};
use uuid::Uuid;

use crate::{bookmark::Bookmark, tag::Tag, user_bookmark_tag::UserBookmarkTag};

#[allow(dead_code)]
pub enum UserBookmark {
    Table,
    Id,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    FolderId,
    UserId,
    BookmarkId,
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
                Self::Title => "title",
                Self::ThumbnailUrl => "thumbnail_url",
                Self::PublishedAt => "published_at",
                Self::Author => "author",
                Self::FolderId => "folder_id",
                Self::UserId => "user_id",
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
    folder_id: Option<Option<Uuid>>,
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
            (UserBookmark::Table, UserBookmark::Title),
            (UserBookmark::Table, UserBookmark::ThumbnailUrl),
            (UserBookmark::Table, UserBookmark::PublishedAt),
            (UserBookmark::Table, UserBookmark::Author),
            (UserBookmark::Table, UserBookmark::FolderId),
            (UserBookmark::Table, UserBookmark::CreatedAt),
        ])
        .columns([(Bookmark::Table, Bookmark::Link)])
        .expr_as(
            Expr::col((Bookmark::Table, Bookmark::Title)),
            Alias::new("original_title"),
        )
        .expr_as(
            Expr::col((Bookmark::Table, Bookmark::ThumbnailUrl)),
            Alias::new("original_thumbnail_url"),
        )
        .expr_as(
            Expr::col((Bookmark::Table, Bookmark::PublishedAt)),
            Alias::new("original_published_at"),
        )
        .expr_as(
            Expr::col((Bookmark::Table, Bookmark::Author)),
            Alias::new("original_author"),
        )
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
        .and_where_option(folder_id.map(|e| {
            e.map_or_else(
                || Expr::col(UserBookmark::FolderId).is_null(),
                |e| Expr::col(UserBookmark::FolderId).eq(e),
            )
        }))
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

pub fn select_by_unique_index(user_id: Uuid, bookmark_id: Uuid) -> SelectStatement {
    Query::select()
        .column(UserBookmark::Id)
        .from(UserBookmark::Table)
        .and_where(Expr::col(UserBookmark::UserId).eq(user_id))
        .and_where(Expr::col(UserBookmark::BookmarkId).eq(bookmark_id))
        .to_owned()
}

#[allow(clippy::too_many_arguments)]
pub fn insert(
    id: Option<Uuid>,
    title: Option<String>,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    folder_id: Option<Uuid>,
    bookmark_id: Uuid,
    user_id: Uuid,
) -> InsertStatement {
    let mut columns = vec![
        UserBookmark::Title,
        UserBookmark::ThumbnailUrl,
        UserBookmark::PublishedAt,
        UserBookmark::Author,
        UserBookmark::FolderId,
        UserBookmark::BookmarkId,
        UserBookmark::UserId,
    ];
    let mut values: Vec<SimpleExpr> = vec![
        title.into(),
        thumbnail_url.into(),
        published_at.into(),
        author.into(),
        folder_id.into(),
        bookmark_id.into(),
        user_id.into(),
    ];

    if let Some(id) = id {
        columns.push(UserBookmark::Id);
        values.push(id.into());
    }

    Query::insert()
        .into_table(UserBookmark::Table)
        .columns(columns)
        .values_panic(values)
        .returning_col(UserBookmark::Id)
        .to_owned()
}

pub fn update(
    id: Uuid,
    title: Option<Option<String>>,
    thumbnail_url: Option<Option<String>>,
    published_at: Option<Option<DateTime<Utc>>>,
    author: Option<Option<String>>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
) -> UpdateStatement {
    let mut query = Query::update()
        .table(UserBookmark::Table)
        .value(UserBookmark::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(UserBookmark::Id).eq(id))
        .and_where(Expr::col(UserBookmark::UserId).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(UserBookmark::Title, title);
    }
    if let Some(thumbnail_url) = thumbnail_url {
        query.value(UserBookmark::ThumbnailUrl, thumbnail_url);
    }
    if let Some(published_at) = published_at {
        query.value(UserBookmark::PublishedAt, published_at);
    }
    if let Some(author) = author {
        query.value(UserBookmark::Author, author);
    }
    if let Some(folder_id) = folder_id {
        query.value(UserBookmark::FolderId, folder_id);
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
