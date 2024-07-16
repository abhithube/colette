use colette_database::feed_entries::InsertParams;
use sqlx::PgExecutor;

pub async fn insert(ex: impl PgExecutor<'_>, params: InsertParams) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
  WITH
       fe AS (
             INSERT INTO feed_entries (feed_id, entry_id)
             VALUES ($1, $2)
                 ON CONFLICT (feed_id, entry_id) DO NOTHING
          RETURNING id
       )
SELECT id AS \"id!\"
  FROM fe
 UNION ALL
SELECT id
  FROM feed_entries
 WHERE feed_id = $1
   AND entry_id = $2",
        params.feed_id,
        params.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
