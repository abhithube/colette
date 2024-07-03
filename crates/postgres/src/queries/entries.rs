use chrono::{DateTime, Utc};
use colette_core::feeds::ProcessedEntry;
use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct InsertData {
    pub link: String,
    pub title: String,
    pub published_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub thumbnail_url: Option<String>,
}

impl From<ProcessedEntry> for InsertData {
    fn from(value: ProcessedEntry) -> Self {
        Self {
            link: value.link.as_str().to_owned(),
            title: value.title,
            published_at: value.published,
            description: value.description,
            author: value.author,
            thumbnail_url: value.thumbnail.map(|e| e.as_str().to_owned()),
        }
    }
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData) -> Result<i32, Error> {
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
