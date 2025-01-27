use colette_core::tag::{Cursor, TagType};

use sqlx::{postgres::PgRow, PgExecutor, Postgres, QueryBuilder};
use uuid::Uuid;

pub async fn select<'a>(
    ex: impl PgExecutor<'a>,
    id: Option<Uuid>,
    user_id: Uuid,
    limit: Option<i64>,
    cursor: Option<Cursor>,
    tag_type: TagType,
) -> sqlx::Result<Vec<PgRow>> {
    let mut qb = QueryBuilder::<Postgres>::new(
        "
SELECT t.id, t.title, count(uft.user_feed_id) AS feed_count, count(ubt.user_bookmark_id) AS bookmark_count
FROM tags t
LEFT JOIN user_feed_tags uft ON uft.tag_id = t.id
LEFT JOIN user_bookmark_tags ubt ON ubt.tag_id = t.id
WHERE t.user_id = ",
    );
    qb.push_bind(user_id);

    if let Some(id) = id {
        qb.push(" AND t.id = ");
        qb.push_bind(id);
    }

    if let Some(cursor) = cursor {
        qb.push(" t.title > ");
        qb.push_bind(cursor.title);
    }

    qb.push(" GROUP BY t.id, t.title");

    match tag_type {
        TagType::Feeds => {
            qb.push(" HAVING count(uft.user_feed_id) > 0");
        }
        TagType::Bookmarks => {
            qb.push(" HAVING count(ubt.user_bookmark_id) > 0");
        }
        _ => {}
    }

    qb.push(" ORDER BY t.title ASC");

    if let Some(limit) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
    }

    qb.build().fetch_all(ex).await
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
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE tags SET ");

    let mut separated = qb.separated(", ");

    if let Some(title) = title {
        separated.push("title = ");
        separated.push_bind_unseparated(title);
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
        "DELETE FROM tags WHERE id = $1 AND user_id = $2",
        id,
        user_id
    )
    .execute(ex)
    .await?;

    Ok(())
}
