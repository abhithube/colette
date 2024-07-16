use colette_database::feed_entries::InsertData;
use sqlx::SqliteExecutor;

#[derive(Debug)]
pub struct SelectParams {
    pub feed_id: i64,
    pub entry_id: i64,
}

pub async fn select(ex: impl SqliteExecutor<'_>, params: SelectParams) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
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

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
   INSERT INTO feed_entries (feed_id, entry_id)
   VALUES ($1, $2)
       ON CONFLICT (feed_id, entry_id) DO NOTHING
RETURNING id",
        data.feed_id,
        data.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
