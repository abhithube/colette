use colette_core::{
    Collection,
    collection::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
        Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: Pool<Postgres>,
}

impl PostgresCollectionRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresCollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let (has_folder, folder_id) = match params.folder_id {
            Some(folder_id) => (true, folder_id),
            None => (false, None),
        };

        let collections = sqlx::query_file_as!(
            Collection,
            "queries/collections/select.sql",
            params.user_id,
            params.id.is_none(),
            params.id,
            !has_folder,
            folder_id,
            params.cursor.is_none(),
            params.cursor.map(|e| e.title),
            params.limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(collections)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresCollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = sqlx::query_file_scalar!(
            "queries/collections/insert.sql",
            data.title.clone(),
            data.folder_id,
            data.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Database(e),
        })?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresCollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() || data.folder_id.is_some() {
            let (has_folder, folder_id) = match data.folder_id {
                Some(folder_id) => (true, folder_id),
                None => (false, None),
            };

            sqlx::query_file!(
                "queries/collections/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title.map(String::from),
                has_folder,
                folder_id
            )
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresCollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/collections/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

impl CollectionRepository for PostgresCollectionRepository {}
