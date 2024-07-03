use colette_core::Feed;
use colette_database::{profile_feeds::InsertData, FindOneParams};
use sqlx::{Error, PgExecutor};

pub async fn insert(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feeds/insert.sql",
        data.profile_id,
        data.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn select_by_id(
    ex: impl PgExecutor<'_>,
    params: FindOneParams<'_>,
) -> Result<Feed, Error> {
    let row = sqlx::query_file_as!(
        Feed,
        "queries/profile_feeds/select_by_id.sql",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
