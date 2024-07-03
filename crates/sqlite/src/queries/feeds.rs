use sqlx::{Error, SqliteExecutor};

#[derive(Debug)]
pub struct InsertData {
    pub link: String,
    pub title: String,
    pub url: Option<String>,
}

pub async fn insert(ex: impl SqliteExecutor<'_>, data: &InsertData) -> Result<i64, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}
