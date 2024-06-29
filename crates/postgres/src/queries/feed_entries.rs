use sqlx::PgExecutor;

use crate::Error;

#[derive(Debug)]
pub struct CreateData {
    pub feed_id: i32,
    pub entry_id: i32,
}

pub async fn insert(ex: impl PgExecutor<'_>, data: &CreateData) -> Result<i32, Error> {
    let row = sqlx::query_file!(
        "queries/feed_entries/insert.sql",
        data.feed_id,
        data.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
