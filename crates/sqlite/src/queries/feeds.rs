use colette_database::feeds::InsertData;
use futures::{Stream, StreamExt};
use sqlx::{Error, SqliteExecutor};

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<i64, Error> {
    let row = sqlx::query_file!("queries/feeds/insert.sql", data.link, data.title, data.url)
        .fetch_one(ex)
        .await?;

    Ok(row.id)
}

pub fn iterate<'a>(
    ex: impl SqliteExecutor<'a> + 'a,
) -> impl Stream<Item = Result<(i64, String), Error>> + 'a {
    sqlx::query_file!("queries/feeds/iterate.sql")
        .fetch(ex)
        .map(|e| e.map(|e| (e.id, e.url)))
}
