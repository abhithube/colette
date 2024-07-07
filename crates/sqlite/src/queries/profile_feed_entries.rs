use colette_core::Entry;
use colette_database::profile_feed_entries::{InsertData, SelectManyParams};
use sqlx::{Error, SqliteExecutor};

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Entry>, Error> {
    let row = sqlx::query_file_as!(
        Entry,
        "queries/profile_feed_entries/select_many.sql",
        params.profile_id,
        params.limit,
        params.published_at,
        params.profile_feed_id,
        params.has_read
    )
    .fetch_all(ex)
    .await?;

    Ok(row)
}

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
