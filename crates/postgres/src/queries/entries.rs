use colette_database::entries::InsertData;
use sqlx::{Error, PgExecutor};

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<i32, Error> {
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
