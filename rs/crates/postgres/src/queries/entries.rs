use colette_database::entries::InsertData;
use sqlx::PgExecutor;

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<i64, sqlx::Error> {
    let row = sqlx::query!(
        "
   INSERT INTO entries (link, title, published_at, description, author, thumbnail_url)
   VALUES ($1, $2, $3, $4, $5, $6)
       ON CONFLICT (link) DO
   UPDATE
      SET title = excluded.title,
          published_at = excluded.published_at,
          description = excluded.description,
          author = excluded.author,
          thumbnail_url = excluded.thumbnail_url
RETURNING id",
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
