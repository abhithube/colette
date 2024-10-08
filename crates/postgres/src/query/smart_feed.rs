use colette_core::smart_feed::Cursor;
use colette_sql::smart_feed;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

use super::smart_feed_filter::build_case_statement;

#[derive(Debug, Clone, sqlx::FromRow)]
struct SmartFeedSelect {
    id: Uuid,
    title: String,
    unread_count: i64,
}

impl From<SmartFeedSelect> for colette_core::SmartFeed {
    fn from(value: SmartFeedSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            unread_count: Some(value.unread_count),
        }
    }
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::SmartFeed>> {
    let query = smart_feed::select(id, profile_id, cursor, limit, build_case_statement());

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, SmartFeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    title: String,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = smart_feed::insert(id, title, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    title: Option<String>,
) -> sqlx::Result<()> {
    let query = smart_feed::update(id, profile_id, title);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, profile_id: Uuid) -> sqlx::Result<()> {
    let query = smart_feed::delete(id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
