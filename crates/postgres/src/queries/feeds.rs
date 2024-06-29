use sqlx::PgExecutor;

use crate::Error;

#[derive(Debug)]
pub struct CreateData {
    pub link: String,
    pub title: String,
    pub url: Option<String>,
}

pub async fn insert(ex: impl PgExecutor<'_>, data: &CreateData) -> Result<i32, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}
