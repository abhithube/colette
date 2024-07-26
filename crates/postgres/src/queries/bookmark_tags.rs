use colette_database::bookmark_tags::SelectParams;
use sqlx::PgExecutor;

pub async fn insert(ex: impl PgExecutor<'_>, params: Vec<SelectParams>) -> Result<(), sqlx::Error> {
    let (bookmark_ids, tag_ids) = params
        .iter()
        .map(|e| (e.bookmark_id, e.tag_id))
        .collect::<(Vec<_>, Vec<_>)>();

    sqlx::query!(
        "
INSERT INTO bookmark_tags (bookmark_id, tag_id)
SELECT * FROM UNNEST($1::UUID[], $2::UUID[])
    ON CONFLICT (bookmark_id, tag_id) DO NOTHING",
        &bookmark_ids,
        &tag_ids,
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete(ex: impl PgExecutor<'_>, params: Vec<SelectParams>) -> Result<(), sqlx::Error> {
    let (bookmark_ids, tag_ids) = params
        .iter()
        .map(|e| (e.bookmark_id, e.tag_id))
        .collect::<(Vec<_>, Vec<_>)>();

    sqlx::query!(
        "
DELETE FROM bookmark_tags
 WHERE (bookmark_id, tag_id) IN (
       SELECT unnest($1::UUID[]) AS bookmark_id,
              unnest($2::UUID[]) AS tag_id
       )",
        &bookmark_ids,
        &tag_ids,
    )
    .execute(ex)
    .await?;

    Ok(())
}
