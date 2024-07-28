use colette_database::bookmark_tags::SelectParams;
use sqlx::SqliteExecutor;

pub async fn insert(ex: impl SqliteExecutor<'_>, params: SelectParams) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
INSERT INTO bookmark_tags (bookmark_id, tag_id)
VALUES ($1, $2)",
        params.bookmark_id,
        params.tag_id,
    )
    .execute(ex)
    .await?;

    Ok(())
}

pub async fn delete(ex: impl SqliteExecutor<'_>, params: SelectParams) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "
DELETE FROM bookmark_tags
 WHERE bookmark_id = $1
   AND tag_id = $2",
        params.bookmark_id,
        params.tag_id,
    )
    .execute(ex)
    .await?;

    Ok(())
}