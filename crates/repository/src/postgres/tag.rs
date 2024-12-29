use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
    Tag,
};
use deadpool_postgres::{
    tokio_postgres::{error::SqlState, Row},
    Pool,
};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: Pool,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresTagRepository {
    type Params = TagFindParams;
    type Output = Result<Vec<Tag>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::tag::select(
            params.id,
            params.user_id,
            params.limit,
            params.cursor,
            params.tag_type,
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::tag::insert(None, data.title.clone(), data.user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = client
            .query_one(&stmt, &values.as_params())
            .await
            .map(|e| e.get::<_, Uuid>("id"))
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title),
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
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let (sql, values) = crate::tag::update(params.id, params.user_id, data.title)
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
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::tag::delete_by_id(params.id, params.user_id)
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

impl TagRepository for PostgresTagRepository {}

#[derive(Debug, Clone)]
struct TagSelect(Tag);

impl From<Row> for TagSelect {
    fn from(value: Row) -> Self {
        Self(Tag {
            id: value.get("id"),
            title: value.get("title"),
            bookmark_count: Some(value.get("bookmark_count")),
            feed_count: Some(value.get("feed_count")),
        })
    }
}
