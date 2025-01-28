use colette_core::{feed_entry::Cursor, FeedEntry};
use sqlx::PgExecutor;
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    user_feed_id: Option<Uuid>,
    has_read: Option<bool>,
    tags: Option<&[String]>,
    cursor: Option<Cursor>,
    limit: Option<i64>,
) -> sqlx::Result<Vec<FeedEntry>> {
    sqlx::query_as!(
        FeedEntry,
        "
SELECT
    ufe.id, ufe.has_read, ufe.user_feed_id AS feed_id,
    fe.link, fe.title, fe.published_at, fe.description, fe.author, fe.thumbnail_url
FROM user_feed_entries ufe
JOIN feed_entries fe ON fe.id = ufe.feed_entry_id
LEFT JOIN user_feed_tags uft ON $1 AND uft.user_feed_id = ufe.user_feed_id
LEFT JOIN tags t ON $1 AND t.id = uft.tag_id AND t.title = any($2)
WHERE NOT $1 OR t.id IS NOT NULL
AND ufe.user_id = $3
AND ($4::bool OR ufe.id = $5)
AND ($6::bool OR ufe.user_feed_id = $7)
AND ($8::bool OR ufe.has_read = $9)
AND ($10::bool OR (fe.published_at, ufe.id) > ($11, $12))
ORDER BY fe.published_at DESC, ufe.id DESC
LIMIT $13",
        tags.is_some(),
        &tags.unwrap_or_default(),
        user_id,
        id.is_none(),
        id,
        user_feed_id.is_none(),
        user_feed_id,
        has_read.is_none(),
        has_read,
        cursor.is_none(),
        cursor.as_ref().map(|e| e.published_at),
        cursor.map(|e| e.id),
        limit,
    )
    .fetch_all(ex)
    .await
}

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    feed_entry_ids: &[Uuid],
    feed_id: Uuid,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
INSERT INTO user_feed_entries (feed_entry_id, user_feed_id, user_id)
SELECT feed_entry_id, uf.id, uf.user_id
FROM UNNEST($1::uuid[]) AS feed_entry_id
JOIN user_feeds uf ON uf.feed_id = $2
ON CONFLICT (user_feed_id, feed_entry_id) DO NOTHING",
        feed_entry_ids,
        feed_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    has_read: Option<bool>,
) -> sqlx::Result<()> {
    sqlx::query!(
        "
UPDATE user_feed_entries
SET
    has_read = CASE WHEN $3 THEN $4 ELSE has_read END,
    updated_at = now()
WHERE id = $1
AND user_id = $2",
        id,
        user_id,
        has_read.is_some(),
        has_read
    )
    .execute(ex)
    .await?;

    Ok(())
}
