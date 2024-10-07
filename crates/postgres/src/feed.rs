use futures::stream::BoxStream;
use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgExecutor;

use crate::profile_feed::ProfileFeed;

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
    let query = Query::select()
        .column(Feed::Id)
        .from(Feed::Table)
        .and_where(
            Expr::col(Feed::Url)
                .eq(url.clone())
                .or(Expr::col(Feed::Link).eq(url)),
        )
        .to_owned();

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
    let query = Query::insert()
        .into_table(Feed::Table)
        .columns([Feed::Link, Feed::Title, Feed::Url])
        .values_panic([link.into(), title.into(), url.into()])
        .on_conflict(
            OnConflict::column(Feed::Link)
                .update_columns([Feed::Title, Feed::Url])
                .to_owned(),
        )
        .returning_col(Feed::Id)
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, i32, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
    let subquery = Query::select()
        .from(ProfileFeed::Table)
        .and_where(
            Expr::col((ProfileFeed::Table, ProfileFeed::FeedId)).equals((Feed::Table, Feed::Id)),
        )
        .to_owned();

    let query = Query::delete()
        .from_table(Feed::Table)
        .and_where(Expr::exists(subquery).not())
        .to_owned();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}

pub fn stream<'a>(
    executor: impl PgExecutor<'a> + 'a,
) -> BoxStream<'a, sqlx::Result<(i32, String)>> {
    sqlx::query_as::<_, (i32, String)>("SELECT id, COALESCE(url, link) FROM feed").fetch(executor)
}
