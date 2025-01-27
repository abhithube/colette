use std::fmt::Write;

use colette_core::tag::{Cursor, TagType};
use sea_query::{Alias, Expr, Iden, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, PgExecutor};
use uuid::Uuid;

use crate::{user_bookmark_tag::UserBookmarkTag, user_feed_tag::UserFeedTag};

#[allow(dead_code)]
pub enum Tag {
    Table,
    Id,
    Title,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Tag {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "tags",
                Self::Id => "id",
                Self::Title => "title",
                Self::UserId => "user_id",
                Self::CreatedAt => "created_at",
                Self::UpdatedAt => "updated_at",
            }
        )
        .unwrap();
    }
}

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    tag_type: TagType,
) -> sqlx::Result<Vec<PgRow>> {
    let mut query = Query::select()
        .column((Tag::Table, Tag::Id))
        .column((Tag::Table, Tag::Title))
        .expr_as(
            Expr::col(UserFeedTag::UserFeedId).count(),
            Alias::new("feed_count"),
        )
        .expr_as(
            Expr::col(UserBookmarkTag::UserBookmarkId).count(),
            Alias::new("bookmark_count"),
        )
        .from(Tag::Table)
        .left_join(
            UserFeedTag::Table,
            Expr::col((UserFeedTag::Table, UserFeedTag::TagId))
                .eq(Expr::col((Tag::Table, Tag::Id))),
        )
        .left_join(
            UserBookmarkTag::Table,
            Expr::col((UserBookmarkTag::Table, UserBookmarkTag::TagId))
                .eq(Expr::col((Tag::Table, Tag::Id))),
        )
        .and_where(Expr::col((Tag::Table, Tag::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((Tag::Table, Tag::Id)).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col(Tag::Title).gt(e.title)))
        .group_by_columns([(Tag::Table, Tag::Id), (Tag::Table, Tag::Title)])
        .order_by((Tag::Table, Tag::Title), Order::Asc)
        .to_owned();

    match tag_type {
        TagType::Bookmarks => {
            query.and_having(Expr::expr(Expr::col(UserBookmarkTag::UserBookmarkId).count()).gt(0));
        }
        TagType::Feeds => {
            query.and_having(Expr::expr(Expr::col(UserFeedTag::UserFeedId).count()).gt(0));
        }
        _ => {}
    };

    if let Some(limit) = limit {
        query.limit(limit);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).fetch_all(ex).await
}

pub async fn select_by_title<'a>(
    ex: impl PgExecutor<'a>,
    title: String,
    user_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT id
FROM tags
WHERE title = $1
AND user_id = $2",
        title,
        user_id
    )
    .fetch_optional(ex)
    .await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    title: String,
    user_id: Uuid,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO tags (title, user_id)
VALUES ($1, $2)
RETURNING id",
        title,
        user_id
    )
    .fetch_one(ex)
    .await
}

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    tags: &[String],
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "INSERT INTO tags (title, user_id)
SELECT *, $2 FROM UNNEST($1::text[])
ON CONFLICT (user_id, title) DO NOTHING",
        tags,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
) -> sqlx::Result<()> {
    let mut query = Query::update()
        .table(Tag::Table)
        .value(Tag::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(Tag::Id).eq(id))
        .and_where(Expr::col(Tag::UserId).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Tag::Title, title);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(ex).await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM tags
WHERE id = $1
AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
