use colette_core::tag::{Cursor, TagFindManyFilters};
use colette_sql::tag::{self, InsertMany};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor, Row};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub(crate) enum Tag {
    Table,
    Id,
    Title,
    ProfileId,
    CreatedAt,
    UpdatedAt,
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TagSelect {
    id: Uuid,
    title: String,
    bookmark_count: i64,
    feed_count: i64,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: Some(value.bookmark_count),
            feed_count: Some(value.feed_count),
        }
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagSelectId {
    pub id: Uuid,
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> sqlx::Result<Vec<colette_core::Tag>> {
    let query = tag::select(id, profile_id, limit, cursor, filters);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_by_title(
    executor: impl PgExecutor<'_>,
    title: String,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = tag::select_by_title(title, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let row = sqlx::query_with(&sql, values).fetch_one(executor).await?;

    row.try_get("id")
}

pub async fn select_ids_by_titles(
    executor: impl PgExecutor<'_>,
    titles: &[String],
    profile_id: Uuid,
) -> sqlx::Result<Vec<Uuid>> {
    let query = tag::select_ids_by_titles(titles, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, TagSelectId, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.id).collect())
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    title: String,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = tag::insert(id, title, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn insert_many(
    executor: impl PgExecutor<'_>,
    data: Vec<InsertMany>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = tag::insert_many(data, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(())
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    title: Option<String>,
) -> sqlx::Result<()> {
    let query = tag::update(id, profile_id, title);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete_by_id(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    let query = tag::delete_by_id(id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete_many(executor: impl PgExecutor<'_>) -> sqlx::Result<u64> {
    let query = tag::delete_many();

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;

    Ok(result.rows_affected())
}
