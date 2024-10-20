use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{
        BookmarkCreateData, BookmarkFindManyFilters, BookmarkRepository, BookmarkUpdateData,
        Cursor, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, TagsLinkAction, TagsLinkData, Updatable},
    Bookmark,
};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Json, SqliteConnection, SqliteExecutor, SqlitePool};
use uuid::Uuid;

pub struct SqliteBookmarkRepository {
    pool: SqlitePool,
}

impl SqliteBookmarkRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Output = Result<Bookmark, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Bookmark, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = {
            let (sql, values) = colette_sql::bookmark::insert(
                data.url,
                data.bookmark.title,
                data.bookmark.thumbnail.map(String::from),
                data.bookmark.published,
                data.bookmark.author,
            )
            .build_sqlx(SqliteQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };

        let pb_id = {
            let (mut sql, mut values) =
                colette_sql::profile_bookmark::select_by_unique_index(data.profile_id, bookmark_id)
                    .build_sqlx(SqliteQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                let id = Uuid::new_v4();

                (sql, values) =
                    colette_sql::profile_bookmark::insert(id, bookmark_id, data.profile_id)
                        .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                id
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pb_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&mut *tx, IdParams::new(pb_id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<Bookmark, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, tags, params.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        let bookmark = find_by_id(&mut *tx, params).await?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(bookmark)
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let count = {
            let (sql, values) = colette_sql::profile_bookmark::delete(params.id, params.profile_id)
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for SqliteBookmarkRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<BookmarkFindManyFilters>,
    ) -> Result<Vec<Bookmark>, Error> {
        find(&self.pool, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct BookmarkSelect {
    pub id: Uuid,
    pub link: String,
    pub title: String,
    pub thumbnail_url: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub author: Option<String>,
    pub created_at: DateTime<Utc>,
    pub tags: Option<Json<Vec<colette_core::Tag>>>,
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
            tags: value.tags.map(|e| e.0.into_iter().collect()),
        }
    }
}

pub(crate) async fn find(
    executor: impl SqliteExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<BookmarkFindManyFilters>,
) -> Result<Vec<Bookmark>, Error> {
    let mut tags: Option<Vec<String>> = None;

    if let Some(filters) = filters {
        tags = filters.tags;
    }

    // let jsonb_agg = Expr::cust(
    //     r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tag"."id"), 'title', "tag"."title") ORDER BY "tag"."title") FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    // );
    let jsonb_agg = Expr::cust(
        r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tag"."id"), 'title', "tag"."title")) FILTER (WHERE "tag"."id" IS NOT NULL)"#,
    );

    let tags_subquery = tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
        )
    });

    let (sql, values) = colette_sql::profile_bookmark::select(
        id,
        profile_id,
        cursor,
        limit,
        jsonb_agg,
        tags_subquery,
    )
    .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with::<_, BookmarkSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(Bookmark::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))
}

pub async fn find_by_id(
    executor: impl SqliteExecutor<'_>,
    params: IdParams,
) -> Result<Bookmark, Error> {
    let mut bookmarks = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if bookmarks.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(bookmarks.swap_remove(0))
}

pub(crate) async fn link_tags(
    conn: &mut SqliteConnection,
    profile_bookmark_id: Uuid,
    tags: TagsLinkData,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    if let TagsLinkAction::Remove = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_in_titles(&tags.data, profile_id)
                .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;

        return Ok(());
    }

    if let TagsLinkAction::Set = tags.action {
        let (sql, values) =
            colette_sql::profile_bookmark_tag::delete_many_not_in_titles(&tags.data, profile_id)
                .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = colette_sql::tag::insert_many(
            tags.data
                .iter()
                .map(|e| colette_sql::tag::InsertMany {
                    id: Uuid::new_v4(),
                    title: e.to_owned(),
                })
                .collect(),
            profile_id,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    let tag_ids = {
        let (sql, values) = colette_sql::tag::select_ids_by_titles(&tags.data, profile_id)
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await?
    };

    {
        let insert_many = tag_ids
            .into_iter()
            .map(|e| colette_sql::profile_bookmark_tag::InsertMany {
                profile_bookmark_id,
                tag_id: e,
            })
            .collect::<Vec<_>>();

        let (sql, values) = colette_sql::profile_bookmark_tag::insert_many(insert_many, profile_id)
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}
