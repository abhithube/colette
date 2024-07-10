use colette_database::feeds::InsertData;
use futures::Stream;
use sqlx::{postgres::PgRow, Error, PgExecutor, Row};

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<i64, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}

pub fn iterate<'a>(
    ex: impl PgExecutor<'a> + 'a,
) -> impl Stream<Item = Result<(i64, String), Error>> + 'a {
    sqlx::query("SELECT id, coalesce(url, link) \"url!: String\" FROM feeds")
        .map(|e: PgRow| (e.get("id"), e.get("url")))
        .fetch(ex)
}
