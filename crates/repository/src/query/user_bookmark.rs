use chrono::{DateTime, Utc};
use colette_core::bookmark::Cursor;
use sqlx::{postgres::PgRow, PgExecutor, Postgres, QueryBuilder};
use uuid::Uuid;

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<i64>,
    tag_titles: Option<Vec<String>>,
) -> sqlx::Result<Vec<PgRow>> {
    let mut qb = QueryBuilder::<Postgres>::new("
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
    jt.tags as tags
FROM user_bookmarks ub
INNER JOIN bookmarks b ON b.id = ub.bookmark_id
LEFT JOIN json_tags jt ON jt.ub_id = ub.id
WHERE ub.user_id = ");

    qb.push_bind(user_id);

    if let Some(id) = id {
        qb.push(" AND ub.id = ");
        qb.push_bind(id);
    }

    if let Some(folder_id) = folder_id {
        if folder_id.is_some() {
            qb.push(" AND ub.folder_id = ");
            qb.push_bind(folder_id);
        } else {
            qb.push(" AND ub.folder_id IS NULL");
        }
    }

    if let Some(tags) = tag_titles {
        qb.push(" AND EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS(jt.tags) AS t WHERE t ->> 'title' IN )");
        qb.push_bind(tags);
    }

    if let Some(cursor) = cursor {
        qb.push(" AND coalesce(ub.published_at, b.published_at) > ");

        qb.push_bind(cursor.created_at);
    }

    qb.push(" ORDER BY coalesce(ub.published_at, b.created_at) ASC");

    if let Some(limit) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
    }

    qb.build().fetch_all(ex).await
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
    title: Option<Option<String>>,
    thumbnail_url: Option<Option<String>>,
    published_at: Option<Option<DateTime<Utc>>>,
    author: Option<Option<String>>,
    folder_id: Option<Option<Uuid>>,
    user_id: Uuid,
) -> sqlx::Result<()> {
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE user_bookmarks SET ");

    let mut separated = qb.separated(", ");

    if let Some(title) = title {
        separated.push("title = ");
        separated.push_bind_unseparated(title);
    }
    if let Some(thumbnail_url) = thumbnail_url {
        separated.push("thumbnail_url = ");
        separated.push_bind_unseparated(thumbnail_url);
    }
    if let Some(published_at) = published_at {
        separated.push("published_at = ");
        separated.push_bind_unseparated(published_at);
    }
    if let Some(author) = author {
        separated.push("author = ");
        separated.push_bind_unseparated(author);
    }
    if let Some(folder_id) = folder_id {
        separated.push("folder_id = ");
        separated.push_bind_unseparated(folder_id);
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
        "DELETE FROM user_bookmarks WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
