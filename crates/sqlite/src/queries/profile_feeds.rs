use colette_core::Feed;
use colette_database::{profile_feeds::UpdateParams, SelectByIdParams, SelectManyParams};
use sqlx::{types::chrono, SqliteExecutor};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SelectParams<'a> {
    pub profile_id: &'a Uuid,
    pub feed_id: i64,
}

#[derive(Clone, Debug)]
pub struct InsertParams<'a> {
    pub id: Uuid,
    pub profile_id: &'a Uuid,
    pub feed_id: i64,
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Feed>, sqlx::Error> {
    let rows = sqlx::query_as!(
        Feed,
        "
SELECT pf.id AS \"id: uuid::Uuid\",
       f.link,
       f.title,
       f.url,
       pf.custom_title,
       pf.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       pf.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\",
       count(pfe.id) AS unread_count
  FROM profile_feeds AS pf
  JOIN feeds AS f
    ON f.id = pf.feed_id
  JOIN feed_entries AS fe
    ON fe.feed_id = f.id
       LEFT JOIN profile_feed_entries AS pfe
       ON pfe.feed_entry_id = fe.id
          AND pfe.has_read = FALSE
 WHERE pf.profile_id = $1
 GROUP BY pf.id, f.link, f.title, f.url
 ORDER BY pf.custom_title ASC, f.title ASC",
        params.profile_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
    params: SelectByIdParams<'_>,
) -> Result<Feed, sqlx::Error> {
    let row = sqlx::query_as!(
        Feed,
        "
SELECT pf.id AS \"id: uuid::Uuid\",
       f.link,
       f.title,
       f.url,
       pf.custom_title,
       pf.created_at AS \"created_at: chrono::DateTime<chrono::Utc>\",
       pf.updated_at AS \"updated_at: chrono::DateTime<chrono::Utc>\",
       count(pfe.id) AS unread_count
  FROM profile_feeds AS pf
  JOIN feeds f
    ON f.id = pf.feed_id
  JOIN feed_entries AS fe
    ON fe.feed_id = f.id
       LEFT JOIN profile_feed_entries AS pfe
       ON pfe.feed_entry_id = fe.id
          AND pfe.has_read = FALSE
 WHERE pf.id = $1
   AND pf.profile_id = $2
 GROUP BY pf.id, f.url, f.link, f.title",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}

pub async fn select(
    ex: impl SqliteExecutor<'_>,
    params: SelectParams<'_>,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        "
SELECT id AS \"id: uuid::Uuid\"
  FROM profile_feeds
 WHERE profile_id = $1
   AND feed_id = $2",
        params.profile_id,
        params.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn insert(
    ex: impl SqliteExecutor<'_>,
    params: InsertParams<'_>,
) -> Result<Uuid, sqlx::Error> {
    let row = sqlx::query!(
        "
   INSERT INTO profile_feeds (id, profile_id, feed_id)
   VALUES ($1, $2, $3)
       ON CONFLICT (profile_id, feed_id) DO NOTHING
RETURNING id AS \"id: uuid::Uuid\"",
        params.id,
        params.profile_id,
        params.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

// try to map pg one to sqlite with claude
pub async fn update(
    ex: impl SqliteExecutor<'_>,
    params: UpdateParams<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
UPDATE profile_feeds
   SET custom_title = $3
 WHERE id = $1
   AND profile_id = $2",
        params.id,
        params.profile_id,
        params.custom_title
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
    params: SelectByIdParams<'_>,
) -> Result<(), sqlx::Error> {
    let result = sqlx::query!(
        "
DELETE FROM profile_feeds
 WHERE id = $1
   AND profile_id = $2",
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
