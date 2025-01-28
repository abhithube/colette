use colette_core::{
    tag::{Cursor, TagType},
    Tag,
};
use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<i64>,
    cursor: Option<Cursor>,
    _tag_type: TagType,
) -> sqlx::Result<Vec<Tag>> {
    sqlx::query_as!(
        Tag,
        "
SELECT t.id, t.title, count(uft.user_feed_id) AS feed_count, count(ubt.user_bookmark_id) AS bookmark_count
FROM tags t
LEFT JOIN user_feed_tags uft ON uft.tag_id = t.id
LEFT JOIN user_bookmark_tags ubt ON ubt.tag_id = t.id
WHERE t.user_id = $1
AND ($2::bool OR t.id = $3)
AND ($4::bool OR t.title > $5)
GROUP BY t.id, t.title
ORDER BY t.title ASC
LIMIT $6",
        user_id,
        id.is_none(),
        id,
        cursor.is_none(),
        cursor.map(|e| e.title),
        limit
    )
    .fetch_all(ex)
    .await
}

pub async fn select_by_title<'a>(
    ex: impl PgExecutor<'a>,
    title: String,
    user_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT id FROM tags WHERE title = $1 AND user_id = $2",
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
        "INSERT INTO tags (title, user_id) VALUES ($1, $2) RETURNING id",
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
        "
INSERT INTO tags (title, user_id)
SELECT *, $2
FROM UNNEST($1::text[])
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
    sqlx::query!(
        "
UPDATE tags
SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    updated_at = now()
WHERE id = $1
AND user_id = $2",
        id,
        user_id,
        title.is_some(),
        title
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM tags WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
