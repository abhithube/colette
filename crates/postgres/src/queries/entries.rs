use sqlx::PgExecutor;
use time::OffsetDateTime;

use crate::Error;

#[derive(Debug)]
pub struct CreateData {
    pub link: String,
    pub title: String,
    pub published_at: OffsetDateTime,
    pub description: String,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

pub async fn insert(ex: impl PgExecutor<'_>, data: &CreateData) -> Result<i32, Error> {
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
