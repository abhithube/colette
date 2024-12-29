use colette_core::{
    bookmark::{
        BookmarkCacheData, BookmarkCreateData, BookmarkFindParams, BookmarkRepository,
        BookmarkUpdateData, Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Bookmark,
};
use deadpool_postgres::{
    tokio_postgres::{self, types::Json, Row},
    GenericClient, Pool,
};
use sea_query::{Expr, ExprTrait, PostgresQueryBuilder};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresBookmarkRepository {
    pool: Pool,
}

impl PostgresBookmarkRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresBookmarkRepository {
    type Params = BookmarkFindParams;
    type Output = Result<Vec<Bookmark>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let jsonb_agg = Expr::cust(
            r#"JSONB_AGG(JSONB_BUILD_OBJECT('id', "tags"."id", 'title', "tags"."title") ORDER BY "tags"."title") FILTER (WHERE "tags"."id" IS NOT NULL)"#,
        );

        let tags_subquery = params.tags.map(|e| {
            Expr::cust_with_expr(
                r#"EXISTS (SELECT 1 FROM JSONB_ARRAY_ELEMENTS("json_tags"."tags") AS "t" WHERE ?)"#,
                Expr::cust(r#""t" ->> 'title'"#).is_in(e),
            )
        });

        let (sql, values) = crate::user_bookmark::select(
            params.id,
            params.user_id,
            params.cursor,
            params.limit,
            jsonb_agg,
            tags_subquery,
        )
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .query(&stmt, &values.as_params())
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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let bookmark_id = {
            let (sql, values) = crate::bookmark::select_by_link(data.url.clone())
                .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            if let Some(row) = tx
                .query_opt(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                Ok(row.get::<_, i32>("id"))
            } else {
                Err(Error::Conflict(data.url))
            }
        }?;

        let pb_id = {
            let (mut sql, mut values) =
                crate::user_bookmark::select_by_unique_index(data.user_id, bookmark_id)
                    .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            if let Some(row) = tx
                .query_opt(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
            {
                row.get::<_, Uuid>("id")
            } else {
                (sql, values) = crate::user_bookmark::insert(None, bookmark_id, data.user_id)
                    .build_postgres(PostgresQueryBuilder);

                let stmt = tx
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                tx.query_one(&stmt, &values.as_params())
                    .await
                    .map(|e| e.get::<_, Uuid>("id"))
                    .map_err(|e| Error::Unknown(e.into()))?
            }
        };

        if let Some(tags) = data.tags {
            link_tags(&tx, pb_id, tags, data.user_id)
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
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if let Some(tags) = data.tags {
            link_tags(&tx, params.id, tags, params.user_id)
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::user_bookmark::delete(params.id, params.user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl BookmarkRepository for PostgresBookmarkRepository {
    async fn cache(&self, data: BookmarkCacheData) -> Result<(), Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::bookmark::insert(
            data.url,
            data.bookmark.title,
            data.bookmark.thumbnail.map(String::from),
            data.bookmark.published,
            data.bookmark.author,
        )
        .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct BookmarkSelect(Bookmark);

impl From<Row> for BookmarkSelect {
    fn from(value: Row) -> Self {
        Self(Bookmark {
            id: value.get("id"),
            link: value.get("link"),
            title: value.get("title"),
            thumbnail_url: value.get("thumbnail_url"),
            published_at: value.get("published_at"),
            author: value.get("author"),
            created_at: value.get("created_at"),
            tags: value
                .get::<_, Option<Json<Vec<colette_core::Tag>>>>("tags")
                .map(|e| e.0),
        })
    }
}

pub(crate) async fn link_tags<C: GenericClient>(
    client: &C,
    user_bookmark_id: Uuid,
    tags: Vec<String>,
    user_id: Uuid,
) -> Result<(), tokio_postgres::Error> {
    {
        let (sql, values) = crate::user_bookmark_tag::delete_many_not_in_titles(&tags, user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
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
            crate::tag::insert_many(&insert_many, user_id).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    {
        let (sql, values) =
            crate::tag::select_ids_by_titles(&tags, user_id).build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        let tag_ids = client.query(&stmt, &values.as_params()).await.map(|e| {
            e.into_iter()
                .map(|e| e.get::<_, Uuid>("id"))
                .collect::<Vec<_>>()
        })?;

        let insert_many = tag_ids
            .into_iter()
            .map(|e| crate::user_bookmark_tag::InsertMany {
                user_bookmark_id,
                tag_id: e,
            })
            .collect::<Vec<_>>();

        let (sql, values) = crate::user_bookmark_tag::insert_many(&insert_many, user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;

        client.execute(&stmt, &values.as_params()).await?;
    }

    Ok(())
}
