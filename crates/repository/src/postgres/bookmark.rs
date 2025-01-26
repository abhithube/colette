use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, ConflictError, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder, WithQuery};
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, types::Json, PgConnection, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: Pool<Postgres>,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresBookmarkRepository {
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = build_select(params).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| BookmarkSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = {
            let (sql, values) =
                crate::bookmark::select_by_link(data.url.clone()).build_sqlx(PostgresQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                Ok(id)
            } else {
                Err(Error::Conflict(ConflictError::NotCached(data.url.clone())))
            }
        }?;

        let pb_id = {
            let (sql, values) = crate::user_bookmark::insert(
                None,
                data.title,
                data.thumbnail_url,
                data.published_at,
                data.author,
                data.folder_id,
                bookmark_id,
                data.user_id,
            )
            .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map(|e| e.get::<Uuid, _>("id"))
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(ConflictError::AlreadyExists(data.url))
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pb_id, &tags, data.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(pb_id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some()
            || data.thumbnail_url.is_some()
            || data.published_at.is_some()
            || data.author.is_some()
            || data.folder_id.is_some()
        {
            let (sql, values) = crate::user_bookmark::update(
                params.id,
                data.title,
                data.thumbnail_url,
                data.published_at,
                data.author,
                data.folder_id,
                params.user_id,
            )
            .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;
        }

        if let Some(tags) = data.tags {
            link_tags(&mut tx, params.id, &tags, params.user_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::user_bookmark::delete(params.id, params.user_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        let (sql, values) = crate::bookmark::insert(
            None,
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) struct BookmarkSelect(pub(crate) Bookmark);

impl From<PgRow> for BookmarkSelect {
    fn from(value: PgRow) -> Self {
        Self(Bookmark {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            thumbnail_url: value.get("thumbnail_url"),
            published_at: value.get("published_at"),
            author: value.get("author"),
            original_title: value.get("original_title"),
            original_thumbnail_url: value.get("original_thumbnail_url"),
            original_published_at: value.get("original_published_at"),
            original_author: value.get("original_author"),
            folder_id: value.get("folder_id"),
            created_at: value.get("created_at"),
            tags: value
                .get::<Option<Json<Vec<colette_core::Tag>>>, _>("tags")
                .map(|e| e.0),
        })
    }
}

pub(crate) fn build_select(params: BookmarkFindParams) -> WithQuery {
    let jsonb_agg = Expr::cust(
        r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tags"."id", 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
    );

    let tags_subquery = params.tags.map(|e| {
        Expr::cust_with_expr(
            r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE ?)"#,
            Expr::cust(r#""t" ->> 'title'"#).is_in(e),
        )
    });

    crate::user_bookmark::select(
        params.id,
        params.folder_id,
        params.user_id,
        params.cursor,
        params.limit,
        jsonb_agg,
        tags_subquery,
    )
}

pub(crate) async fn link_tags(
    conn: &mut PgConnection,
    user_bookmark_id: Uuid,
    tags: &[String],
    user_id: Uuid,
) -> Result<(), sqlx::Error> {
    {
        let (sql, values) = crate::user_bookmark_tag::delete_many_not_in_titles(tags, user_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let insert_many = tags
            .iter()
            .map(|e| crate::tag::InsertMany {
                id: Some(Uuid::new_v4()),
                title: e.to_owned(),
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::tag::insert_many(&insert_many, user_id).build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = crate::user_bookmark_tag::insert_many(user_bookmark_id, tags, user_id)
            .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}
