use colette_database::feeds::InsertData;
use sqlx::{Error, PgExecutor};

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<i64, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}
