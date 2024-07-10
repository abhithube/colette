use colette_database::feeds::InsertData;
use futures::Stream;
use sqlx::{sqlite::SqliteRow, Error, Row, SqliteExecutor};

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<i64, Error> {
    let row = sqlx::query!(
        "
   INSERT INTO feeds (link, title, url)
   VALUES ($1, $2, $3)
       ON CONFLICT (link) DO
   UPDATE
      SET title = excluded.title,
          url = excluded.url
RETURNING id",
        data.link,
        data.title,
        data.url
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub fn iterate<'a>(
    ex: impl SqliteExecutor<'a> + 'a,
) -> impl Stream<Item = Result<(i64, String), Error>> + 'a {
    sqlx::query(
        "
SELECT id,
       coalesce(url, link) AS \"url!: String\"
  FROM feeds",
    )
    .map(|e: SqliteRow| (e.get("id"), e.get("url")))
    .fetch(ex)
}
