use colette_core::folder::Cursor;
use sqlx::{postgres::PgRow, PgExecutor, Postgres, QueryBuilder};
use uuid::Uuid;

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    parent_id: Option<Option<Uuid>>,
    limit: Option<i64>,
    cursor: Option<Cursor>,
) -> sqlx::Result<Vec<PgRow>> {
    let mut qb =
        QueryBuilder::<Postgres>::new("SELECT id, title, parent_id FROM folders WHERE user_id = ");
    qb.push_bind(user_id);

    if let Some(id) = id {
        qb.push(" AND id = ");
        qb.push_bind(id);
    }
    if let Some(parent_id) = parent_id {
        if parent_id.is_some() {
            qb.push(" AND parent_id = ");
            qb.push_bind(parent_id);
        } else {
            qb.push(" AND parent_id IS NULL");
        }
    }

    if let Some(cursor) = cursor {
        qb.push(" title > ");
        qb.push_bind(cursor.title);
    }

    qb.push(" ORDER BY title ASC");

    if let Some(limit) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
    }

    qb.build().fetch_all(ex).await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    title: String,
    parent_id: Option<Uuid>,
    user_id: Uuid,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "INSERT INTO folders (title, parent_id, user_id) VALUES ($1, $2, $3) RETURNING id",
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
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE folders SET ");

    let mut separated = qb.separated(", ");

    if let Some(title) = title {
        separated.push("title = ");
        separated.push_bind_unseparated(title);
    }
    if let Some(parent_id) = parent_id {
        separated.push("parent_id = ");
        separated.push_bind_unseparated(parent_id);
    }

    separated.push("updated_at = now()");

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(" AND user_id = ");
    qb.push_bind(user_id);

    qb.build().execute(ex).await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM folders WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
