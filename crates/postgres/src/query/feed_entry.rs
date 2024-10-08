use colette_sql::feed_entry::{self, InsertMany};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgExecutor;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum FeedEntry {
    Table,
    Id,
    Link,
    Title,
    PublishedAt,
    Description,
    Author,
    ThumbnailUrl,
    FeedId,
    CreatedAt,
    UpdatedAt,
}

pub async fn select_many_by_feed_id(
    executor: impl PgExecutor<'_>,
    feed_id: i32,
) -> sqlx::Result<Vec<i32>> {
    let query = feed_entry::select_many_by_feed_id(feed_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_all(executor)
        .await
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    feed_id: i32,
) -> sqlx::Result<()> {
    let query = feed_entry::insert_many(data, feed_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
    let query = feed_entry::delete_many();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}
