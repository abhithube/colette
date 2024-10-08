use colette_sql::feed;
use futures::stream::BoxStream;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::PgExecutor;

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Feed {
    Table,
    Id,
    Link,
    Title,
    Url,
    CreatedAt,
    UpdatedAt,
}

pub async fn select_by_url(executor: impl PgExecutor<'_>, url: String) -> sqlx::Result<i32> {
    let query = feed::select_by_url(url);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    link: String,
    title: String,
    url: Option<String>,
) -> sqlx::Result<i32> {
    let query = feed::insert(link, title, url);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
    let query = feed::delete_many();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}

pub fn stream<'a>(
    executor: impl PgExecutor<'a> + 'a,
) -> BoxStream<'a, sqlx::Result<(i32, String)>> {
    sqlx::query_as::<_, (i32, String)>("SELECT id, COALESCE(url, link) FROM feed").fetch(executor)
}
