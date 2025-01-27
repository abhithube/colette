use sqlx::PgExecutor;
use uuid::Uuid;

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    user_bookmark_id: Uuid,
    titles: &[String],
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query_scalar!(
        "
INSERT INTO user_bookmark_tags (user_bookmark_id, tag_id, user_id)
SELECT $1, id, user_id
FROM tags
WHERE user_id = $3
AND title = ANY($2)
ON CONFLICT (user_bookmark_id, tag_id) DO NOTHING",
        user_bookmark_id,
        titles,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete_many<'a>(
    ex: impl PgExecutor<'a>,
    titles: &[String],
    user_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
DELETE FROM user_bookmark_tags
WHERE user_id = $1
AND tag_id IN (
    SELECT id FROM tags WHERE user_id = $1 AND title = ANY($2)
)",
        user_id,
        titles
    )
    .execute(ex)
    .await?;

    Ok(())
}
