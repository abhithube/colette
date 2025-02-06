use colette_core::{
    collection::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
        Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Collection,
};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: Pool,
}

impl PostgresCollectionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresCollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) =
            crate::collection::select(params.id, params.user_id, params.limit, params.cursor)
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
                    .map(|e| CollectionSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresCollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::collection::insert(None, data.title.clone(), data.user_id)
            .build_postgres(PostgresQueryBuilder);

        let stmt = client
            .prepare_cached(&sql)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = client
            .query_one(&stmt, &values.as_params())
            .await
            .map(|e| e.get::<_, Uuid>("id"))
            .map_err(|e| Error::Unknown(e.into()))?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresCollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let (sql, values) = crate::collection::update(params.id, params.user_id, data.title)
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
impl Deletable for PostgresCollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::collection::delete_by_id(params.id, params.user_id)
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

impl CollectionRepository for PostgresCollectionRepository {}

#[derive(Debug, Clone)]
struct CollectionSelect(Collection);

impl From<Row> for CollectionSelect {
    fn from(value: Row) -> Self {
        Self(Collection {
            id: value.get("id"),
            title: value.get("title"),
        })
    }
}
