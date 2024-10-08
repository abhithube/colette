use colette_core::feed::Cursor;
use colette_sql::profile_feed;
use sea_query::{Expr, PostgresQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{
    types::{Json, Uuid},
    PgExecutor,
};

#[derive(Debug, Clone, sqlx::FromRow)]
struct FeedSelect {
    pub id: Uuid,
    pub link: String,
    pub title: Option<String>,
    pub pinned: bool,
    pub original_title: String,
    pub url: Option<String>,
    pub tags: Option<Json<Vec<TagSelect>>>,
    pub unread_count: i64,
}

impl From<FeedSelect> for colette_core::Feed {
    fn from(value: FeedSelect) -> Self {
        Self {
            id: value.id,
            link: value.link,
            title: value.title,
            pinned: value.pinned,
            original_title: value.original_title,
            url: value.url,
            tags: value
                .tags
                .map(|e| e.0.into_iter().map(|e| e.into()).collect()),
            unread_count: Some(value.unread_count),
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
    pinned: Option<bool>,
    tags: Option<Vec<String>>,
    cursor: Option<Cursor>,
    limit: Option<u64>,
) -> sqlx::Result<Vec<colette_core::Feed>> {
    let jsonb_agg = Expr::cust(
        r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tag"."id", 'title', "tag"."title") ORDER BY "tag"."title") FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    );

    let tags_subquery = tags.map(|e| {
        Expr::cust_with_expr(r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE "t" ->> 'title' = ANY($1))"#, e)
    });

    let query = profile_feed::select(
        id,
        profile_id,
        pinned,
        cursor,
        limit,
        jsonb_agg,
        tags_subquery,
    );

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_as_with::<_, FeedSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(|e| e.into()).collect())
}

pub async fn select_by_unique_index(
    executor: impl PgExecutor<'_>,
    profile_id: Uuid,
    feed_id: i32,
) -> sqlx::Result<Uuid> {
    let query = profile_feed::select_by_unique_index(profile_id, feed_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn insert(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    pinned: Option<bool>,
    feed_id: i32,
    profile_id: Uuid,
) -> sqlx::Result<Uuid> {
    let query = profile_feed::insert(id, pinned, feed_id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
        .fetch_one(executor)
        .await
}

pub async fn update(
    executor: impl PgExecutor<'_>,
    id: Uuid,
    profile_id: Uuid,
    title: Option<Option<String>>,
    pinned: Option<bool>,
) -> sqlx::Result<()> {
    let query = profile_feed::update(id, profile_id, title, pinned);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn delete(executor: impl PgExecutor<'_>, id: Uuid, profile_id: Uuid) -> sqlx::Result<()> {
    let query = profile_feed::delete(id, profile_id);

    let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
    let result = sqlx::query_with(&sql, values).execute(executor).await?;
    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}
