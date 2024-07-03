use colette_core::Feed;
use sqlx::{Error, PgExecutor};

use super::FindOneParams;

#[derive(Debug)]
pub struct InsertData<'a> {
    pub profile_id: &'a str,
    pub feed_id: i32,
}

pub async fn create(ex: impl PgExecutor<'_>, data: InsertData<'_>) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feeds/create.sql",
        data.profile_id,
        data.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn find_one(ex: impl PgExecutor<'_>, params: FindOneParams<'_>) -> Result<Feed, Error> {
    let row = sqlx::query_file_as!(
        Feed,
        "queries/profile_feeds/find_one.sql",
        params.id,
        params.profile_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row)
}
