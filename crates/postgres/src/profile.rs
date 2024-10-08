use colette_core::profile::Cursor;
use colette_sql::profile;
use futures::stream::BoxStream;
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

#[derive(Debug, Clone, sqlx::FromRow)]
struct ProfileSelect {
    id: Uuid,
    title: String,
    image_url: Option<String>,
    is_default: bool,
    user_id: Uuid,
}

impl From<ProfileSelect> for colette_core::Profile {
    fn from(value: ProfileSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            image_url: value.image_url,
            is_default: value.is_default,
            user_id: value.user_id,
        }
    }
}

pub async fn select(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    user_id: Uuid,
    is_default: Option<bool>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::Profile>> {
    let query = profile::select(id, user_id, is_default, cursor, limit);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    title: String,
    image_url: Option<String>,
    is_default: Option<bool>,
    user_id: Uuid,
) -> sqlx::Result<colette_core::Profile> {
    let query = profile::insert(id, title, image_url, is_default, user_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    user_id: Uuid,
    title: Option<String>,
    image_url: Option<Option<String>>,
) -> sqlx::Result<colette_core::Profile> {
    let query = profile::update(id, user_id, title, image_url);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    let query = profile::delete(id, user_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub fn stream<'a>(
    executor: impl PgExecutor<'a> + 'a,
    feed_id: i32,
) -> BoxStream<'a, sqlx::Result<Uuid>> {
    sqlx::query_scalar::<_, Uuid>("SELECT DISTINCT profile_id FROM profile_feed WHERE feed_id = $1")
        .bind(feed_id)
        .fetch(executor)
}
