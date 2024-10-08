use colette_sql::bookmark;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::chrono::{DateTime, Utc},
    PgExecutor,
};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Bookmark {
    Table,
    Id,
    Link,
    Title,
    ThumbnailUrl,
    PublishedAt,
    Author,
    CreatedAt,
    UpdatedAt,
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    link: String,
    title: String,
    thumbnail_url: Option<String>,
    published_at: Option<DateTime<Utc>>,
    author: Option<String>,
) -> sqlx::Result<i32> {
    let query = bookmark::insert(link, title, thumbnail_url, published_at, author);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_one(executor)
        .await
}
