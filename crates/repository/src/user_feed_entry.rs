use colette_core::feed_entry::Cursor;
use sqlx::{postgres::PgRow, PgExecutor, Postgres, QueryBuilder};
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
) -> sqlx::Result<Vec<PgRow>> {
    let mut qb = QueryBuilder::<Postgres>::new(
        "
SELECT
    ufe.id, ufe.has_read, ufe.user_feed_id,
    fe.link, fe.title, fe.published_at, fe.description, fe.author, fe.thumbnail_url
FROM user_feed_entries ufe
JOIN feed_entries fe ON fe.id = ufe.feed_entry_id",
    );

    if let Some(tags) = tags {
        qb.push(
            " 
INNER JOIN user_feed_tags uft ON uft.user_feed_id = ufe.user_feed_id
INNER JOIN tags t ON t.id = uft.tag_id WHERE title IN (",
        );

        let mut separated = qb.separated(", ");
        for title in tags {
            separated.push_bind(title);
        }

        separated.push_unseparated(")");
    }

    qb.push(" WHERE user_id = ");
    qb.push_bind(user_id);

    if let Some(id) = id {
        qb.push(" AND id = ");
        qb.push_bind(id);
    }

    if let Some(user_feed_id) = user_feed_id {
        qb.push(" AND ufe.user_feed_id = ");
        qb.push_bind(user_feed_id);
    }

    if let Some(has_read) = has_read {
        qb.push(" AND ufe.has_read = ");
        qb.push_bind(has_read);
    }

    if let Some(cursor) = cursor {
        qb.push(" (fe.published_at, ufe.id) > (");

        let mut separated = qb.separated(", ");
        separated.push_bind(cursor.published_at);
        separated.push_bind(cursor.id);

        separated.push_unseparated(")");
    }

    qb.push(" ORDER BY fe.published_at DESC, ufe.id DESC");

    if let Some(limit) = limit {
        qb.push(" LIMIT ");
        qb.push_bind(limit);
    }

    qb.build().fetch_all(ex).await
}

pub async fn insert_many<'a>(
    ex: impl PgExecutor<'a>,
    feed_entry_ids: &[Uuid],
    feed_id: Uuid,
) -> sqlx::Result<()> {
    let mut qb = QueryBuilder::<Postgres>::new(
        "
INSERT INTO user_feed_entries (feed_entry_id, user_feed_id, user_id)
SELECT feed_entry_id, uf.id, uf.user_id FROM UNNEST(",
    );
    qb.push_bind(feed_entry_ids);
    qb.push(") AS feed_entry_id JOIN user_feeds uf ON uf.feed_id = ");
    qb.push_bind(feed_id);

    qb.push(" ON CONFLICT (user_feed_id, feed_entry_id) DO NOTHING");

    qb.build().execute(ex).await?;

    Ok(())
}

pub async fn update<'a>(
    ex: impl PgExecutor<'a>,
    id: Uuid,
    user_id: Uuid,
    has_read: Option<bool>,
) -> sqlx::Result<()> {
    let mut qb: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE user_feed_entries SET ");

    let mut separated = qb.separated(", ");

    if let Some(has_read) = has_read {
        separated.push("has_read = ");
        separated.push_bind_unseparated(has_read);
    }

    separated.push("updated_at = now()");

    qb.push(" WHERE id = ");
    qb.push_bind(id);
    qb.push(" AND user_id = ");
    qb.push_bind(user_id);

    qb.build().execute(ex).await?;

    Ok(())
}
