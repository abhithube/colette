use colette_database::profile_feed_tags::SelectParams;
use sqlx::PgExecutor;

pub async fn insert(ex: impl PgExecutor<'_>, params: Vec<SelectParams>) -> Result<(), sqlx::Error> {
    let (profile_feed_ids, tag_ids) = params
        .iter()
        .map(|e| (e.profile_feed_id, e.tag_id))
        .collect::<(Vec<_>, Vec<_>)>();

    sqlx::query!(
        "
INSERT INTO profile_feed_tags (profile_feed_id, tag_id)
SELECT * FROM UNNEST($1::UUID[], $2::UUID[])
    ON CONFLICT (profile_feed_id, tag_id) DO NOTHING",
        &profile_feed_ids,
        &tag_ids,
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete(ex: impl PgExecutor<'_>, params: Vec<SelectParams>) -> Result<(), sqlx::Error> {
    let (profile_feed_ids, tag_ids) = params
        .iter()
        .map(|e| (e.profile_feed_id, e.tag_id))
        .collect::<(Vec<_>, Vec<_>)>();

    sqlx::query!(
        "
DELETE FROM profile_feed_tags
 WHERE (profile_feed_id, tag_id) IN (
       SELECT unnest($1::UUID[]) AS profile_feed_id,
              unnest($2::UUID[]) AS tag_id
       )",
        &profile_feed_ids,
        &tag_ids,
    )
    .execute(ex)
    .await?;

    Ok(())
}
