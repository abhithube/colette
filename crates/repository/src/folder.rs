use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
    Folder,
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresFolderRepository {
    pool: Pool<Postgres>,
}

impl PostgresFolderRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresFolderRepository {
    type Params = FolderFindParams;
    type Output = Result<Vec<Folder>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let folders = crate::common::select_folders(
            &self.pool,
            params.id,
            params.user_id,
            params.parent_id,
            params.limit,
            params.cursor,
        )
        .await?;

        Ok(folders)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresFolderRepository {
    type Data = FolderCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let id = sqlx::query_file_scalar!(
            "queries/folders/insert.sql",
            String::from(data.title),
            data.parent_id,
            data.user_id
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFolderRepository {
    type Params = IdParams;
    type Data = FolderUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() || data.parent_id.is_some() {
            let (has_parent, parent_id) = match data.parent_id {
                Some(parent_id) => (true, parent_id),
                None => (false, None),
            };

            sqlx::query_file!(
                "queries/folders/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title.map(String::from),
                has_parent,
                parent_id
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
impl Deletable for PostgresFolderRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        sqlx::query_file!("queries/folders/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Database(e),
            })?;

        Ok(())
    }
}

impl FolderRepository for PostgresFolderRepository {}
