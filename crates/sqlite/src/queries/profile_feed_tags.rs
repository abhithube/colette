use colette_database::profile_feed_tags::SelectParams;
use sqlx::SqliteExecutor;

pub async fn insert(ex: impl SqliteExecutor<'_>, params: SelectParams) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
 INSERT INTO profile_feed_tags (profile_feed_id, tag_id)
 VALUES ($1, $2)",
        params.profile_feed_id,
        params.tag_id,
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete(ex: impl SqliteExecutor<'_>, params: SelectParams) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
DELETE FROM profile_feed_tags
 WHERE profile_feed_id = $1
   AND tag_id = $2",
        params.profile_feed_id,
        params.tag_id,
    )
    .execute(ex)
    .await?;

    Ok(())
}
