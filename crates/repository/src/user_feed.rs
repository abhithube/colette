use colette_core::feed::Cursor;
use sqlx::{postgres::PgRow, PgExecutor, Postgres, QueryBuilder};
use uuid::Uuid;

#[allow(clippy::too_many_arguments)]
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
WITH unread_counts AS (
    SELECT uf.id as uf_id, count(ufe.id) as count
    FROM user_feeds uf
    INNER JOIN user_feed_entries ufe ON ufe.user_feed_id = uf.id
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
    jt.tags as tags,
    coalesce(uc.count, 0) AS unread_count
FROM user_feeds uf
INNER JOIN feeds f ON f.id = uf.feed_id
LEFT JOIN json_tags jt ON jt.uf_id = uf.id
LEFT JOIN unread_counts uc ON uc.uf_id = uf.id
WHERE uf.user_id = ");

    qb.push_bind(user_id);

    if let Some(id) = id {
        qb.push(" AND uf.id = ");
        qb.push_bind(id);
    }

    if let Some(folder_id) = folder_id {
        if folder_id.is_some() {
            qb.push(" AND uf.folder_id = ");
            qb.push_bind(folder_id);
        } else {
            qb.push(" AND uf.folder_id IS NULL");
        }
    }

    if let Some(tags) = tag_titles {
        qb.push(" AND EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS(jt.tags) AS t WHERE t ->> 'title' IN )");
        qb.push_bind(tags);
    }

    if let Some(cursor) = cursor {
        qb.push(" AND (coalesce(uf.title, f.title), uf.id) < (");

        let mut separated = qb.separated(", ");
        separated.push_bind(cursor.title);
        separated.push_bind(cursor.id);

        separated.push_unseparated(")");
    }

    qb.push(" ORDER BY coalesce(uf.title, f.title) ASC, uf.id ASC");

    if let Some(limit) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
    }

    qb.build().fetch_all(ex).await
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
    let mut qb = QueryBuilder::<Postgres>::new("UPDATE user_feeds SET ");

    let mut separated = qb.separated(", ");

    if let Some(title) = title {
        separated.push("title = ");
        separated.push_bind_unseparated(title);
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
        "DELETE FROM user_feeds WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
