use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct InsertData {
    pub feed_id: i32,
    pub entry_id: i32,
}

pub async fn insert(ex: impl PgExecutor<'_>, data: &InsertData) -> Result<i32, Error> {
    let row = sqlx::query_file!(
        "queries/feed_entries/insert.sql",
        data.feed_id,
        data.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}