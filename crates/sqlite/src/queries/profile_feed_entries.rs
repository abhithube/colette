use colette_database::profile_feed_entries::InsertData;
use sqlx::{Error, SqliteExecutor};

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feed_entries/insert.sql",
        data.id,
        data.profile_feed_id,
        data.feed_entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
