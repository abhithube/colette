use colette_core::{feed::Cursor, Feed, Tag};
use sqlx::{types::Json, PgExecutor};
use uuid::Uuid;

pub struct FeedRow {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub xml_url: Option<String>,
    pub original_title: String,
    pub folder_id: Option<Uuid>,
    pub tags: Option<Json<Vec<Tag>>>,
    pub unread_count: Option<i64>,
}

impl From<FeedRow> for Feed {
    fn from(value: FeedRow) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            xml_url: value.xml_url,
            original_title: value.original_title,
            folder_id: value.folder_id,
            tags: value.tags.map(|e| e.0),
            unread_count: value.unread_count,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<i64>,
    tags: Option<Vec<String>>,
) -> sqlx::Result<Vec<Feed>> {
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    sqlx::query_as!(
        FeedRow,
        r#"
WITH unread_counts AS (
    SELECT uf.id as uf_id, count(ufe.id) as count
    FROM user_feeds uf
    INNER JOIN user_feed_entries ufe ON ufe.user_feed_id = uf.id AND NOT ufe.has_read
    GROUP BY uf.id
),
json_tags AS (
    SELECT
        uf.id AS uf_id,
        jsonb_agg(jsonb_build_object('id', t.id, 'title', t.title) ORDER BY t.title) FILTER (WHERE t.id IS NOT NULL) as tags
    FROM user_feeds uf
    INNER JOIN user_feed_tags uft ON uft.user_feed_id = uf.id
    INNER JOIN tags t ON t.id = uft.tag_id
    GROUP BY uf.id
)
SELECT
    uf.id, uf.title, uf.folder_id,
    f.link, f.title AS original_title, f.xml_url,
    jt.tags as "tags: Json<Vec<Tag>>",
    coalesce(uc.count, 0) AS unread_count
FROM user_feeds uf
INNER JOIN feeds f ON f.id = uf.feed_id
LEFT JOIN json_tags jt ON jt.uf_id = uf.id
LEFT JOIN unread_counts uc ON uc.uf_id = uf.id
WHERE uf.user_id = $1
AND ($2::bool OR uf.id = $3)
AND ($4::bool OR CASE WHEN $5::uuid IS NULL THEN uf.folder_id IS NULL ELSE uf.folder_id = $5 END)
AND ($6::bool OR EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS(jt.tags) AS t WHERE t ->> 'title' = any($7)))
AND ($8::bool OR (coalesce(uf.title, f.title), uf.id) > ($9, $10))
ORDER BY coalesce(uf.title, f.title) ASC, uf.id ASC
LIMIT $11"#,
        user_id,
        id.is_none(),
        id,
        !has_folder,
        folder_id,
        tags.is_none(),
        &tags.unwrap_or_default(),
        cursor.is_none(),
        cursor.as_ref().map(|e| e.title.clone()),
        cursor.map(|e| e.id),
        limit
    )
    .fetch_all(ex)
    .await
    .map(|e| e.into_iter().map(Feed::from)
    .collect())
}

pub async fn select_by_unique_index<'a>(
    ex: impl PgExecutor<'a>,
    user_id: Uuid,
    feed_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT id FROM user_feeds WHERE user_id = $1 AND feed_id = $2",
        user_id,
        feed_id
    )
    .fetch_optional(ex)
    .await
}

pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    title: Option<String>,
    folder_id: Option<Uuid>,
    feed_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "
INSERT INTO user_feeds (title, folder_id, feed_id, user_id)
VALUES ($1, $2, $3, $4)
RETURNING id",
        title,
        folder_id,
        feed_id,
        user_id
    )
    .fetch_one(ex)
    .await
}

pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    title: Option<Option<String>>,
    folder_id: Option<Option<Uuid>>,
) -> sqlx::Result<()> {
    let (has_title, title) = match title {
        Some(title) => (true, title),
        None => (false, None),
    };
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    sqlx::query!(
        "
UPDATE user_feeds
SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    folder_id = CASE WHEN $5 THEN $6 ELSE folder_id END,
    updated_at = now()
WHERE id = $1
AND user_id = $2",
        id,
        user_id,
        has_title,
        title,
        has_folder,
        folder_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM user_feeds WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
