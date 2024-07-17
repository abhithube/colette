use colette_core::Bookmark;
use colette_database::{
    bookmarks::{SelectManyParams, UpdateParams},
    FindOneParams,
};
use sqlx::PgExecutor;

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Bookmark>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Bookmark,
        "
SELECT b.id,
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at,
       b.author,
       b.custom_title,
       b.custom_thumbnail_url,
       b.custom_published_at,
       b.custom_author,
       CASE
       WHEN c.is_default THEN NULL
       ELSE b.collection_id
       END AS collection_id,
       b.created_at,
       b.updated_at
  FROM bookmarks AS b
  JOIN collections AS c
    ON c.id = b.collection_id
 WHERE c.profile_id = $1
   AND ($3::timestamptz IS NULL OR b.published_at < $3)
   AND CASE
       WHEN $4::boolean THEN (
            CASE
            WHEN $5::uuid IS NULL THEN c.is_default
            ELSE b.collection_id = $5
            END
       )
       ELSE TRUE
       END
 ORDER BY b.custom_published_at DESC, b.published_at DESC, b.custom_title ASC, b.title ASC
 LIMIT $2",
        params.profile_id,
        params.limit,
        params.published_at,
        params.should_filter,
        params.collection_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn update(
    ex: impl PgExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<Bookmark, sqlx::Error> {
    let row = sqlx::query_as!(
        Bookmark,
        "
   UPDATE bookmarks AS b
      SET custom_title = coalesce($3, custom_title),
          custom_thumbnail_url = coalesce($4, custom_thumbnail_url),
          custom_published_at = coalesce($5, custom_published_at),
          custom_author = coalesce($6, custom_author)
     FROM collections AS c
    WHERE b.id = $1
      AND c.profile_id = $2
RETURNING b.id,
          b.link,
          b.title,
          b.thumbnail_url,
          b.published_at,
          b.author,
          b.custom_title,
          b.custom_thumbnail_url,
          b.custom_published_at,
          b.custom_author,
          CASE
          WHEN c.is_default THEN NULL
          ELSE b.collection_id
          END AS collection_id,
          b.created_at,
          b.updated_at",
        params.id,
        params.profile_id,
        params.custom_title,
        params.custom_thumbnail_url,
        params.custom_published_at,
        params.custom_author
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn delete(ex: impl PgExecutor<'_>, params: FindOneParams<'_>) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
DELETE FROM bookmarks AS b
 USING collections AS c
 WHERE b.id = $1
   AND b.collection_id = c.id
   AND c.profile_id = $2",
        params.id,
        params.profile_id
    )
    .execute(ex)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
