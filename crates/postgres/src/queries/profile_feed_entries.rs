use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct CreateData {
    pub profile_feed_id: String,
    pub feed_entry_id: i32,
}

pub async fn create(ex: impl PgExecutor<'_>, data: CreateData) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feed_entries/create.sql",
        data.profile_feed_id,
        data.feed_entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
