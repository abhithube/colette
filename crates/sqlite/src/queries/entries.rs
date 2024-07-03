use chrono::{DateTime, Utc};
use sqlx::{Error, SqliteExecutor};

#[derive(Debug)]
pub struct InsertData {
    pub link: String,
    pub title: String,
    pub published_at: DateTime<Utc>,
    pub description: String,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub async fn insert(ex: impl SqliteExecutor<'_>, data: &InsertData) -> Result<i64, Error> {
    let row = sqlx::query_file!(
        "queries/entries/insert.sql",
        data.link,
        data.title,
        data.published_at,
        data.description,
        data.author,
        data.thumbnail_url
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
