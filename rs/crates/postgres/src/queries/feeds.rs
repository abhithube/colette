use colette_core::common::SendableStream;
use colette_database::feeds::InsertParams;
use sqlx::{postgres::PgRow, PgExecutor, Row};

pub async fn insert(ex: impl PgExecutor<'_>, params: InsertParams<'_>) -> Result<i64, sqlx::Error> {
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
    ex: impl PgExecutor<'a> + 'a,
) -> SendableStream<'a, Result<(i64, String), sqlx::Error>> {
    Box::pin(
        sqlx::query(
            "
SELECT id,
       coalesce(url, link) AS url
  FROM feeds",
        )
        .map(|e: PgRow| (e.get(0), e.get(1)))
        .fetch(ex),
    )
}
