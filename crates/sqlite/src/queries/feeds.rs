use colette_core::common::SendableStream;
use colette_database::feeds::InsertParams;
use sqlx::{sqlite::SqliteRow, Row, SqliteExecutor};

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
   INSERT INTO feeds (link, title, url)
   VALUES ($1, $2, $3)
       ON CONFLICT (link) DO
   UPDATE
      SET title = excluded.title,
          url = excluded.url
RETURNING id",
        params.link,
        params.title,
        params.url
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub fn iterate<'a>(
    ex: impl SqliteExecutor<'a> + 'a,
) -> SendableStream<'a, Result<(i64, String), sqlx::Error>> {
    sqlx::query(
        "
SELECT id,
       coalesce(url, link) AS url
  FROM feeds",
    )
    .map(|e: SqliteRow| (e.get(0), e.get(1)))
    .fetch(ex)
}

pub async fn cleanup(ex: impl SqliteExecutor<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
DELETE FROM feeds AS f
 WHERE NOT EXISTS (
       SELECT 1
         FROM profile_feeds AS pf
        WHERE pf.feed_id = f.id
 )"
    )
    .execute(ex)
    .await?;

    Ok(())
}