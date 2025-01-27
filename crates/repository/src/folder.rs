use std::fmt::Write;

use colette_core::folder::Cursor;
use sea_query::{Expr, Iden, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, PgExecutor};
use uuid::Uuid;

#[allow(dead_code)]
pub enum Folder {
    Table,
    Id,
    Title,
    ParentId,
    UserId,
    CreatedAt,
    UpdatedAt,
}

impl Iden for Folder {
    fn unquoted(&self, s: &mut dyn Write) {
        write!(
            s,
            "{}",
            match self {
                Self::Table => "folders",
                Self::Id => "id",
                Self::Title => "title",
                Self::ParentId => "parent_id",
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
    parent_id: Option<Option<Uuid>>,
    limit: Option<u64>,
    cursor: Option<Cursor>,
) -> sqlx::Result<Vec<PgRow>> {
    let mut query = Query::select()
        .columns([Folder::Id, Folder::Title, Folder::ParentId])
        .from(Folder::Table)
        .and_where(Expr::col(Folder::UserId).eq(user_id))
        .and_where_option(id.map(|e| Expr::col(Folder::Id).eq(e)))
        .and_where_option(parent_id.map(|e| {
            e.map_or_else(
                || Expr::col(Folder::ParentId).is_null(),
                |e| Expr::col(Folder::ParentId).eq(e),
            )
        }))
        .and_where_option(cursor.map(|e| Expr::col(Folder::Title).gt(e.title)))
        .order_by((Folder::Table, Folder::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        query.limit(limit);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).fetch_all(ex).await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    title: String,
    parent_id: Option<Uuid>,
    user_id: Uuid,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO folders (title, parent_id, user_id)
VALUES ($1, $2, $3)
RETURNING id",
        title,
        parent_id,
        user_id
    )
    .fetch_one(ex)
    .await
}

pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    parent_id: Option<Option<Uuid>>,
) -> sqlx::Result<()> {
    let mut query = Query::update()
        .table(Folder::Table)
        .value(Folder::UpdatedAt, Expr::current_timestamp())
        .and_where(Expr::col(Folder::Id).eq(id))
        .and_where(Expr::col(Folder::UserId).eq(user_id))
        .to_owned();

    if let Some(title) = title {
        query.value(Folder::Title, title);
    }
    if let Some(parent_id) = parent_id {
        query.value(Folder::ParentId, parent_id);
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values).execute(ex).await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM folders
WHERE id = $1
AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
