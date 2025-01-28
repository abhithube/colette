use colette_core::folder::Cursor;
use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    parent_id: Option<Option<Uuid>>,
    limit: Option<i64>,
    cursor: Option<Cursor>,
) -> sqlx::Result<Vec<colette_core::Folder>> {
    let (has_parent, parent_id) = match parent_id {
        Some(parent_id) => (true, parent_id),
        None => (false, None),
    };

    sqlx::query_as!(
        colette_core::Folder,
        "
SELECT id, title, parent_id
FROM folders
WHERE user_id = $1
AND ($2::bool OR id = $3)
AND ($4::bool OR CASE WHEN $5::uuid IS NULL THEN parent_id IS NULL ELSE parent_id = $5 END)
AND ($6::bool OR title > $7)
ORDER BY title ASC
LIMIT $8",
        user_id,
        id.is_none(),
        id,
        !has_parent,
        parent_id,
        cursor.is_none(),
        cursor.map(|e| e.title),
        limit
    )
    .fetch_all(ex)
    .await
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
    let (has_parent, parent_id) = match parent_id {
        Some(parent_id) => (true, parent_id),
        None => (false, None),
    };

    sqlx::query!(
        "
UPDATE folders
SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    parent_id = CASE WHEN $5 THEN $6 ELSE parent_id END,
    updated_at = now()
WHERE id = $1
AND user_id = $2",
        id,
        user_id,
        title.is_some(),
        title,
        has_parent,
        parent_id
    )
    .execute(ex)
    .await?;

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
