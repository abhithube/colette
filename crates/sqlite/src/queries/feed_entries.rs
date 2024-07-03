use sqlx::{Error, SqliteExecutor};

#[derive(Debug)]
pub struct SelectParams {
    pub feed_id: i32,
    pub entry_id: i32,
}

#[derive(Debug)]
pub struct InsertData {
    pub feed_id: i32,
    pub entry_id: i32,
}

pub async fn select(ex: impl SqliteExecutor<'_>, params: &SelectParams) -> Result<i64, Error> {
    let row = sqlx::query_file!(
        "queries/feed_entries/select.sql",
        params.feed_id,
        params.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn insert(ex: impl SqliteExecutor<'_>, data: &InsertData) -> Result<i64, Error> {
    let row = sqlx::query_file!(
        "queries/feed_entries/insert.sql",
        data.feed_id,
        data.entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
