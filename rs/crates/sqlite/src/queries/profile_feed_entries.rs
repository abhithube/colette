use colette_core::Entry;
use colette_database::profile_feed_entries::SelectManyParams;
use sqlx::{Error, SqliteExecutor};
use uuid::Uuid;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub id: Uuid,
    pub profile_feed_id: &'a Uuid,
    pub feed_entry_id: i64,
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Entry>, Error> {
    let row = sqlx::query_as!(
        Entry,
        "
SELECT pfe.id AS \"id: uuid::Uuid\",
       e.link,
       e.title,
       e.published_at AS \"published_at: chrono::DateTime<chrono::Utc>\",
       e.description,
       e.author,
       e.thumbnail_url,
       pfe.has_read AS \"has_read: bool\",
       pfe.profile_feed_id AS \"feed_id: uuid::Uuid\"
  FROM profile_feed_entries AS pfe
  JOIN profile_feeds AS pf
    ON pf.id = pfe.profile_feed_id
  JOIN feed_entries AS fe
    ON fe.id = pfe.feed_entry_id
  JOIN entries AS e
    ON e.id = fe.entry_id
 WHERE pf.profile_id = $1
   AND ($3 IS NULL OR e.published_at < $3)
   AND ($4 IS NULL OR pfe.profile_feed_id = $4)
   AND ($5 IS NULL OR pfe.has_read = $5)
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

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<Uuid, Error> {
    let row = sqlx::query!(
        "
   INSERT INTO profile_feed_entries (id, profile_feed_id, feed_entry_id)
   VALUES ($1, $2, $3)
       ON CONFLICT (profile_feed_id, feed_entry_id) DO NOTHING
RETURNING id AS \"id: uuid::Uuid\"",
        data.id,
        data.profile_feed_id,
        data.feed_entry_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
