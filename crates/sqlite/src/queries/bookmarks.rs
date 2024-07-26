use colette_core::Bookmark;
use colette_database::{
    bookmarks::{SelectManyParams, UpdateParams},
    SelectByIdParams,
};
use sqlx::{
    types::{
        chrono::{self, DateTime, Utc},
        Uuid,
    },
    SqliteExecutor,
};

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub id: Uuid,
    pub link: &'a str,
    pub title: &'a str,
    pub thumbnail_url: Option<&'a str>,
    pub published_at: Option<&'a DateTime<Utc>>,
    pub author: Option<&'a str>,
    pub collection_id: Uuid,
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Bookmark>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Bookmark,
        "
SELECT b.id AS \"id: uuid::Uuid\",
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at AS \"published_at: chrono::DateTime<chrono::Utc>\",
       b.author,
       b.custom_title,
       b.custom_thumbnail_url,
       b.custom_published_at AS \"custom_published_at: chrono::DateTime<chrono::Utc>\",
       b.custom_author,
       CASE
       WHEN c.is_default THEN NULL
       ELSE b.collection_id
       END AS \"collection_id: uuid::Uuid\",
       b.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       b.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM bookmarks AS b
  JOIN collections AS c
    ON c.id = b.collection_id
 WHERE c.profile_id = $1
   AND ($3 IS NULL OR b.published_at < $3)
   AND CASE
       WHEN $4 THEN (
            CASE
            WHEN $5 IS NULL THEN c.is_default
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

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams,
) -> Result<Bookmark, sqlx::Error> {
    let row = sqlx::query_as!(
        Bookmark,
        "
SELECT b.id AS \"id: uuid::Uuid\",
       b.link,
       b.title,
       b.thumbnail_url,
       b.published_at AS \"published_at: chrono::DateTime<chrono::Utc>\",
       b.author,
       b.custom_title,
       b.custom_thumbnail_url,
       b.custom_published_at AS \"custom_published_at: chrono::DateTime<chrono::Utc>\",
       b.custom_author,
       CASE
       WHEN c.is_default THEN NULL
       ELSE b.collection_id
       END AS \"collection_id: uuid::Uuid\",
       b.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       b.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\"
  FROM bookmarks AS b
  JOIN collections AS c
    ON c.id = b.collection_id
 WHERE b.id = $1
   AND c.profile_id = $2",
        params.id,
        params.profile_id,
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
   INSERT INTO bookmarks AS b (id, link, title, thumbnail_url, published_at, author, collection_id)
   VALUES ($1, $2, $3, $4, $5, $6, $7)
       ON CONFLICT (collection_id, link) DO
   UPDATE
      SET title = coalesce(excluded.title, b.title),
          thumbnail_url = coalesce(excluded.thumbnail_url, b.thumbnail_url),
          published_at = coalesce(excluded.published_at, b.published_at),
          author = coalesce(excluded.author, b.author)",
        params.id,
        params.link,
        params.title,
        params.thumbnail_url,
        params.published_at,
        params.author,
        params.collection_id,
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn update(
    ex: impl SqliteExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query_as!(
        Bookmark,
        "
UPDATE bookmarks AS b
   SET custom_title = coalesce($3, custom_title),
       custom_thumbnail_url = coalesce($4, custom_thumbnail_url),
       custom_published_at = coalesce($5, custom_published_at),
       custom_author = coalesce($6, custom_author)
  FROM collections AS c
 WHERE b.id = $1
   AND c.profile_id = $2",
        params.id,
        params.profile_id,
        params.custom_title,
        params.custom_thumbnail_url,
        params.custom_published_at,
        params.custom_author
    )
    .execute(ex)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
DELETE FROM bookmarks
 WHERE id = $1
   AND collection_id IN (
       SELECT id
         FROM collections
        WHERE profile_id = $2
       )",
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
