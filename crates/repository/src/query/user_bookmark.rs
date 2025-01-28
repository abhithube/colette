use chrono::{DateTime, Utc};
use colette_core::{bookmark::Cursor, Bookmark, Tag};
use sqlx::{types::Json, PgExecutor};
use uuid::Uuid;

pub struct BookmarkRow {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub original_title: String,
    pub original_thumbnail_url: Option<String>,
    pub original_published_at: Option<DateTime<Utc>>,
    pub original_author: Option<String>,
    pub folder_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Json<Vec<Tag>>>,
}

impl From<BookmarkRow> for Bookmark {
    fn from(value: BookmarkRow) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            original_title: value.original_title,
            original_thumbnail_url: value.original_thumbnail_url,
            original_published_at: value.original_published_at,
            original_author: value.original_author,
            folder_id: value.folder_id,
            created_at: value.created_at,
            tags: value.tags.map(|e| e.0),
        }
    }
}

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<i64>,
    tags: Option<Vec<String>>,
) -> sqlx::Result<Vec<Bookmark>> {
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    sqlx::query_as!(
        BookmarkRow,
        r#"
WITH json_tags AS (
    SELECT
        ub.id AS ub_id,
        jsonb_agg(jsonb_build_object('id', t.id, 'title', t.title) ORDER BY t.title) FILTER (WHERE t.id IS NOT NULL) as tags
    FROM user_bookmarks ub
    INNER JOIN user_bookmark_tags ubt ON ubt.user_bookmark_id = ub.id
    INNER JOIN tags t ON t.id = ubt.tag_id
    GROUP BY ub.id
)
SELECT
    ub.id, ub.title, ub.thumbnail_url, ub.published_at, ub.author, ub.folder_id, ub.created_at,
    b.link, b.title AS original_title, b.thumbnail_url AS original_thumbnail_url, b.published_at AS original_published_at, b.author AS original_author,
    jt.tags as "tags: Json<Vec<Tag>>"
FROM user_bookmarks ub
INNER JOIN bookmarks b ON b.id = ub.bookmark_id
LEFT JOIN json_tags jt ON jt.ub_id = ub.id
WHERE ub.user_id = $1
AND ($2::bool OR ub.id = $3)
AND ($4::bool OR CASE WHEN $5::uuid IS NULL THEN ub.folder_id IS NULL ELSE ub.folder_id = $5 END)
AND ($6::bool OR EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS(jt.tags) AS t WHERE t ->> 'title' = any($7)))
AND ($8::bool OR coalesce(ub.created_at, b.created_at) > $9)
ORDER BY coalesce(ub.published_at, b.created_at) ASC
LIMIT $10"#,
        user_id,
        id.is_none(),
        id,
        !has_folder,
        folder_id,
        tags.is_none(),
        &tags.unwrap_or_default(),
        cursor.is_none(),
        cursor.map(|e| e.created_at),
        limit
    )
    .fetch_all(ex)
    .await
    .map(|e| e.into_iter().map(Bookmark::from)
    .collect())
}

pub async fn select_by_unique_index<'a>(
    ex: impl PgExecutor<'a>,
    user_id: Uuid,
    bookmark_id: Uuid,
) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        "SELECT id FROM user_bookmarks WHERE user_id = $1 AND bookmark_id = $2",
        user_id,
        bookmark_id
    )
    .fetch_optional(ex)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert<'a>(
    ex: impl PgExecutor<'a>,
    title: Option<String>,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
    folder_id: Option<Uuid>,
    bookmark_id: Uuid,
    user_id: Uuid,
) -> sqlx::Result<Uuid> {
    sqlx::query_scalar!(
        "
INSERT INTO user_bookmarks (title, thumbnail_url, published_at, author, folder_id, bookmark_id, user_id)
VALUES ($1, $2, $3, $4, $5, $6, $7)
RETURNING id",
        title,
        thumbnail_url,
        published_at,
        author,
        folder_id,
        bookmark_id,
        user_id
    )
    .fetch_one(ex)
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    title: Option<Option<String>>,
    thumbnail_url: Option<Option<String>>,
    published_at: Option<Option<DateTime<Utc>>>,
    author: Option<Option<String>>,
    folder_id: Option<Option<Uuid>>,
) -> sqlx::Result<()> {
    let (has_title, title) = match title {
        Some(title) => (true, title),
        None => (false, None),
    };
    let (has_thumbnail_url, thumbnail_url) = match thumbnail_url {
        Some(thumbnail_url) => (true, thumbnail_url),
        None => (false, None),
    };
    let (has_published_at, published_at) = match published_at {
        Some(published_at) => (true, published_at),
        None => (false, None),
    };
    let (has_author, author) = match author {
        Some(author) => (true, author),
        None => (false, None),
    };
    let (has_folder, folder_id) = match folder_id {
        Some(folder_id) => (true, folder_id),
        None => (false, None),
    };

    sqlx::query!(
        "
UPDATE user_bookmarks
SET
    title = CASE WHEN $3 THEN $4 ELSE title END,
    thumbnail_url = CASE WHEN $5 THEN $6 ELSE thumbnail_url END,
    published_at = CASE WHEN $7 THEN $8 ELSE published_at END,
    author = CASE WHEN $9 THEN $10 ELSE author END,
    folder_id = CASE WHEN $11 THEN $12 ELSE folder_id END,
    updated_at = now()
WHERE id = $1
AND user_id = $2",
        id,
        user_id,
        has_title,
        title,
        has_thumbnail_url,
        thumbnail_url,
        has_published_at,
        published_at,
        has_author,
        author,
        has_folder,
        folder_id
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete<'a>(ex: impl PgExecutor<'a>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    sqlx::query!(
        "DELETE FROM user_bookmarks WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
