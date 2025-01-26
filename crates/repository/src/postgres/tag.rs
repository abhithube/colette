use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
    Tag,
};
use sea_query::PostgresQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: Pool<Postgres>,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresTagRepository {
    type Params = TagFindParams;
    type Output = Result<Vec<Tag>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (sql, values) = crate::tag::select(
            params.id,
            params.user_id,
            params.limit,
            params.cursor,
            params.tag_type,
        )
        .build_sqlx(PostgresQueryBuilder);

        sqlx::query_with(&sql, values)
            .fetch_all(&self.pool)
            .await
            .map(|e| {
                e.into_iter()
                    .map(|e| TagSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresTagRepository {
    type Data = TagCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let (sql, values) = crate::tag::insert(None, data.title.clone(), data.user_id)
            .build_sqlx(PostgresQueryBuilder);

        let id = sqlx::query_with(&sql, values)
            .fetch_one(&self.pool)
            .await
            .map(|e| e.get::<Uuid, _>("id"))
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            let (sql, values) = crate::tag::update(params.id, params.user_id, data.title)
                .build_sqlx(PostgresQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => Error::NotFound(params.id),
                    _ => Error::Unknown(e.into()),
                })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let (sql, values) =
            crate::tag::delete_by_id(params.id, params.user_id).build_sqlx(PostgresQueryBuilder);

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

impl TagRepository for PostgresTagRepository {}

#[derive(Debug, Clone)]
struct TagSelect(Tag);

impl From<PgRow> for TagSelect {
    fn from(value: PgRow) -> Self {
        Self(Tag {
            id: value.get("id"),
            title: value.get("title"),
            bookmark_count: Some(value.get("bookmark_count")),
            feed_count: Some(value.get("feed_count")),
        })
    }
}
