use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
    Folder,
};
use deadpool_postgres::{tokio_postgres::Row, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresFolderRepository {
    pool: Pool,
}

impl PostgresFolderRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFolderRepository {
    type Params = FolderFindParams;
    type Output = Result<Vec<Folder>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::folder::select(
            params.id,
            params.user_id,
            params.parent_id,
            params.limit,
            params.cursor,
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
                    .map(|e| FolderSelect::from(e).0)
                    .collect::<Vec<_>>()
            })
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFolderRepository {
    type Data = FolderCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) =
            crate::folder::insert(None, data.title.clone(), data.parent_id, data.user_id)
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
impl Updatable for PostgresFolderRepository {
    type Params = IdParams;
    type Data = FolderUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let (sql, values) =
                crate::folder::update(params.id, params.user_id, data.title, data.parent_id)
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
impl Deletable for PostgresFolderRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let (sql, values) = crate::folder::delete_by_id(params.id, params.user_id)
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

impl FolderRepository for PostgresFolderRepository {}

#[derive(Debug, Clone)]
pub(crate) struct FolderSelect(pub(crate) Folder);

impl From<Row> for FolderSelect {
    fn from(value: Row) -> Self {
        Self(Folder {
            id: value.get("id"),
            title: value.get("title"),
            parent_id: value.get("parent_id"),
        })
    }
}
