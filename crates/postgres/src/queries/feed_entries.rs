use colette_database::feed_entries::InsertData;
use sqlx::{Error, PgExecutor};

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData) -> Result<i64, Error> {
    let row = sqlx::query_file!(
        "queries/feed_entries/insert.sql",
        data.feed_id,
        data.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
