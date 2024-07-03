use sqlx::{Error, PgExecutor};

#[derive(Debug)]
pub struct InsertData<'a> {
    pub link: &'a str,
    pub title: &'a str,
    pub url: Option<&'a str>,
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<i32, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}
