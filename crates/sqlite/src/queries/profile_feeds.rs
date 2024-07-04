use colette_core::Feed;
use colette_database::{
    profile_feeds::{InsertData, SelectManyParams},
    FindOneParams,
};
use sqlx::{Error, SqliteExecutor};

#[derive(Debug)]
pub struct SelectParams<'a> {
    pub profile_id: &'a str,
    pub feed_id: i64,
}

pub async fn select_many(
    ex: impl SqliteExecutor<'_>,
    params: SelectManyParams<'_>,
) -> Result<Vec<Feed>, Error> {
    let rows = sqlx::query_file_as!(
        Feed,
        "queries/profile_feeds/select_many.sql",
        params.profile_id
    )
    .fetch_all(ex)
    .await?;

    Ok(rows)
}

pub async fn select_by_id(
    ex: impl SqliteExecutor<'_>,
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

pub async fn select(
    ex: impl SqliteExecutor<'_>,
    params: SelectParams<'_>,
) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feeds/select.sql",
        params.profile_id,
        params.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}

pub async fn insert(ex: impl SqliteExecutor<'_>, data: InsertData<'_>) -> Result<String, Error> {
    let row = sqlx::query_file!(
        "queries/profile_feeds/insert.sql",
        data.id,
        data.profile_id,
        data.feed_id
    )
    .fetch_one(ex)
    .await?;

    Ok(row.id)
}
