use colette_database::entries::InsertParams;
use sqlx::SqliteExecutor;

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
   INSERT INTO entries (link, title, published_at, description, author, thumbnail_url)
   VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (link) DO
   UPDATE
      SET title = excluded.title,
          published_at = excluded.published_at,
          description = excluded.description,
          author = excluded.author,
          thumbnail_url = excluded.thumbnail_url
RETURNING id",
        params.link,
        params.title,
        params.published_at,
        params.description,
        params.author,
        params.thumbnail_url
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn cleanup(ex: impl SqliteExecutor<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
DELETE FROM entries AS e
 WHERE NOT EXISTS (
       SELECT 1
         FROM feed_entries AS fe
        WHERE fe.entry_id = e.id
 )"
    )
    .execute(ex)
    .await?;

    Ok(())
}
