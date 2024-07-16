use colette_core::Entry;
use colette_database::profile_feed_entries::SelectManyParams;
use sqlx::{types::Uuid, Error, PgExecutor};

#[derive(Debug)]
pub struct InsertData<'a> {
    pub profile_feed_id: &'a Uuid,
    pub feed_entry_id: i64,
}

pub async fn select_many(
    ex: impl PgExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Entry>, Error> {
    let row = sqlx::query_as!(
        Entry,
        "
SELECT pfe.id,
       e.link,
       e.title,
       e.published_at,
       e.description,
       e.author,
       e.thumbnail_url,
       pfe.has_read,
       pfe.profile_feed_id AS feed_id
  FROM profile_feed_entries AS pfe
  JOIN profile_feeds AS pf
    ON pf.id = pfe.profile_feed_id
  JOIN feed_entries AS fe
    ON fe.id = pfe.feed_entry_id
  JOIN entries AS e
    ON e.id = fe.entry_id
 WHERE pf.profile_id = $1
   AND ($3::timestamptz IS NULL OR e.published_at < $3)
   AND ($4::uuid IS NULL OR pfe.profile_feed_id = $4)
   AND ($5::boolean IS NULL OR pfe.has_read = $5)
 ORDER BY e.published_at DESC, pfe.id DESC
 LIMIT $2",
        params.profile_id,
        params.limit,
        params.published_at,
        params.profile_feed_id,
        params.has_read
    )
    .fetch_all(ex)
    .await?;

    Ok(row)
}

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<Uuid, Error> {
    let row = sqlx::query!(
        "
  WITH
       pfe AS (
              INSERT INTO profile_feed_entries (profile_feed_id, feed_entry_id)
              VALUES ($1, $2)
                  ON CONFLICT (profile_feed_id, feed_entry_id) DO
             NOTHING
           RETURNING id
       )
SELECT id AS \"id!\"
  FROM pfe
 UNION ALL
SELECT id
  FROM profile_feed_entries
 WHERE profile_feed_id = $1
   AND feed_entry_id = $2",
        data.profile_feed_id,
        data.feed_entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
