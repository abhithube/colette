use colette_core::profile::Cursor;
use futures::stream::BoxStream;
use sea_query::{Expr, Func, Order, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Uuid, PgExecutor};

#[allow(dead_code)]
#[derive(sea_query::Iden)]
pub enum Profile {
    Table,
    Id,
    Title,
    ImageUrl,
    IsDefault,
    UserId,
    CreatedAt,
    UpdatedAt,
}

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
    let mut query = Query::select()
        .columns([
            Profile::Id,
            Profile::Title,
            Profile::ImageUrl,
            Profile::IsDefault,
            Profile::UserId,
        ])
        .from(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .and_where_option(id.map(|e| Expr::col((Profile::Table, Profile::Id)).eq(e)))
        .and_where_option(is_default.map(|e| Expr::col((Profile::Table, Profile::IsDefault)).eq(e)))
        .and_where_option(cursor.map(|e| Expr::col((Profile::Table, Profile::Title)).gt(e.title)))
        .order_by((Profile::Table, Profile::Title), Order::Asc)
        .to_owned();

    if let Some(limit) = limit {
        query.limit(limit);
    }

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
    let query = Query::insert()
        .into_table(Profile::Table)
        .columns([
            Profile::Id,
            Profile::Title,
            Profile::ImageUrl,
            Profile::IsDefault,
            Profile::UserId,
        ])
        .values_panic([
            id.into(),
            title.into(),
            image_url.into(),
            is_default.unwrap_or_default().into(),
            user_id.into(),
        ])
        .returning_all()
        .to_owned();

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
    let mut query = Query::update()
        .table(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::Id)).eq(id))
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .returning_all()
        .to_owned();

    if let Some(title) = title {
        query.value(
            Profile::Title,
            Func::coalesce([
                title.into(),
                Expr::col((Profile::Table, Profile::Title)).into(),
            ]),
        );
    }
    if let Some(image_url) = image_url {
        query.value(
            Profile::ImageUrl,
            Func::coalesce([
                image_url.into(),
                Expr::col((Profile::Table, Profile::ImageUrl)).into(),
            ]),
        );
    }

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, ProfileSelect, _>(&sql, values)
        .fetch_one(executor)
        .await
        .map(|e| e.into())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, user_id: Uuid) -> sqlx::Result<()> {
    let query = Query::delete()
        .from_table(Profile::Table)
        .and_where(Expr::col((Profile::Table, Profile::Id)).eq(id))
        .and_where(Expr::col((Profile::Table, Profile::UserId)).eq(user_id))
        .to_owned();

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
