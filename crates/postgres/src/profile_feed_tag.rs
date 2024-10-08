use colette_sql::profile_feed_tag::{self, InsertMany};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

#[derive(sea_query::Iden)]
pub enum ProfileFeedTag {
    Table,
    ProfileFeedId,
    TagId,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = profile_feed_tag::insert_many(data, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_in_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = profile_feed_tag::delete_many_in_titles(titles, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn delete_many_not_in_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = profile_feed_tag::delete_many_not_in_titles(titles, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}
