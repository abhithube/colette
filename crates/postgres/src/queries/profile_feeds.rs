use colette_core::Feed;
use colette_database::{
    profile_feeds::{InsertData, SelectManyParams},
    FindOneParams,
};
use sqlx::{Error, PgExecutor};

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Feed>, Error> {
    let rows = sqlx::query_as!(
        Feed,
        "
SELECT pf.id,
       f.link,
       f.title,
       f.url,
       pf.custom_title,
       pf.created_at,
       pf.updated_at,
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
    ex: impl PgExecutor<'_>,
    params: FindOneParams<'_>,
) -> Result<Feed, Error> {
    let row = sqlx::query_as!(
        Feed,
        "
SELECT pf.id,
       f.link,
       f.title,
       f.url,
       pf.custom_title,
       pf.created_at,
       pf.updated_at,
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

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<String, Error> {
    let row = sqlx::query!(
        "
  WITH
       pf AS (
             INSERT INTO profile_feeds (id, profile_id, feed_id)
             VALUES ($1, $2, $3)
                 ON CONFLICT (profile_id, feed_id) DO NOTHING
          RETURNING id
       )
SELECT id AS \"id!\"
  FROM pf
 UNION ALL
SELECT id
  FROM profile_feeds
 WHERE profile_id = $2
   AND feed_id = $3",
        data.id,
        data.profile_id,
        data.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn delete(ex: impl PgExecutor<'_>, params: FindOneParams<'_>) -> Result<(), Error> {
    sqlx::query!(
        "
DELETE FROM profile_feeds
 WHERE id = $1
   AND profile_id = $2",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(())
}
