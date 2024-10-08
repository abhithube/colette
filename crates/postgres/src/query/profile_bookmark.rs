use colette_core::bookmark::Cursor;
use colette_sql::profile_bookmark;
use sea_query::{Expr, PostgresQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Json, Uuid,
    },
    PgExecutor,
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct BookmarkSelect {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Json<Vec<TagSelect>>>,
}

impl From<BookmarkSelect> for colette_core::Bookmark {
    fn from(value: BookmarkSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            thumbnail_url: value.thumbnail_url,
            published_at: value.published_at,
            author: value.author,
            created_at: value.created_at,
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(|e| e.into()).collect()),
        }
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
struct TagSelect {
    id: Uuid,
    title: String,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: None,
            feed_count: None,
        }
    }
}

pub async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    tags: Option<Vec<String>>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::Bookmark>> {
    let jsonb_agg = Expr::cust(
        r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tag"."id", 'title', "tag"."title") ORDER BY "tag"."title") FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    );

    let tags_subquery = tags.map(|e| {
        Expr::cust_with_expr(r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE "t" ->> 'title' = ANY($1))"#, e)
    });

    let query = profile_bookmark::select(id, profile_id, cursor, limit, jsonb_agg, tags_subquery);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, BookmarkSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_by_unique_index(
    executor: impl PgExecutor<'_>,
    profile_id: Uuid,
    bookmark_id: i32,
) -> sqlx::Result<Uuid> {
    let query = profile_bookmark::select_by_unique_index(profile_id, bookmark_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    bookmark_id: i32,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = profile_bookmark::insert(id, bookmark_id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, profile_id: Uuid) -> sqlx::Result<()> {
    let query = profile_bookmark::delete(id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
