use chrono::{DateTime, Utc};
use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use sea_query::{Expr, ExprTrait, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use sqlx::{types::Json, SqliteConnection, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone)]
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
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let jsonb_agg = Expr::cust(
            r#"JSON_GROUP_ARRAY(JSON_OBJECT('id', HEX("tags"."id"), 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
        );

        let tags_subquery = params.tags.map(|e| {
            Expr::cust_with_expr(
                r#"EXISTS (SELECT 1 FROM JSON_EACH("json_tags"."tags") AS "t" WHERE ?)"#,
                Expr::cust(r#""t"."value" ->> 'title'"#).is_in(e),
            )
        });

        let (sql, values) = crate::profile_bookmark::select(
            params.id,
            params.profile_id,
            params.cursor,
            params.limit,
            jsonb_agg,
            tags_subquery,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with::<_, BookmarkSelect, _>(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| e.into_iter().map(Bookmark::from).collect::<Vec<_>>())
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteBookmarkRepository {
    type Data = BookmarkCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = {
            let (sql, values) = crate::bookmark::select_by_link(data.url.clone())
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_scalar_with::<_, i32, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::Conflict(data.url),
                    _ => Error::Unknown(e.into()),
                })?
        };

        let pb_id = {
            let (mut sql, mut values) =
                crate::profile_bookmark::select_by_unique_index(data.profile_id, bookmark_id)
                    .build_sqlx(SqliteQueryBuilder);

            if let Some(id) = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_optional(&mut *tx)
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                id
            } else {
                (sql, values) = crate::profile_bookmark::insert(
                    Some(Uuid::new_v4()),
                    bookmark_id,
                    data.profile_id,
                )
                .build_sqlx(SqliteQueryBuilder);

                sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                    .fetch_one(&mut *tx)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&mut tx, pb_id, tags, data.profile_id)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;
        }

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(pb_id)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Data = BookmarkUpdateData;
    type Output = Result<(), Error>;

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

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteBookmarkRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let count = {
            let (sql, values) = crate::profile_bookmark::delete(params.id, params.profile_id)
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
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        let (sql, values) = crate::bookmark::insert(
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
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

pub(crate) async fn link_tags(
    conn: &mut SqliteConnection,
    profile_bookmark_id: Uuid,
    tags: Vec<String>,
    profile_id: Uuid,
) -> sqlx::Result<()> {
    {
        let (sql, values) =
            crate::profile_bookmark_tag::delete_many_not_in_titles(&tags, profile_id)
                .build_sqlx(SqliteQueryBuilder);

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
            crate::tag::insert_many(&insert_many, profile_id).build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    {
        let (sql, values) = crate::tag::select_ids_by_titles(&tags, profile_id)
            .build_sqlx(SqliteQueryBuilder);

        let tag_ids = sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
            .fetch_all(&mut *conn)
            .await?;

        let insert_many = tag_ids
            .into_iter()
            .map(|e| crate::profile_bookmark_tag::InsertMany {
                profile_bookmark_id,
                tag_id: e,
            })
            .collect::<Vec<_>>();

        let (sql, values) =
            crate::profile_bookmark_tag::insert_many(&insert_many, profile_id)
                .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values).execute(&mut *conn).await?;
    }

    Ok(())
}
