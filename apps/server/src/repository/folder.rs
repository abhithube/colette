use colette_core::{
    Folder,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
};
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::common;

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
        let folders = common::select_folders(
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
            data.title,
            FolderType::from(data.folder_type) as FolderType,
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
                data.title,
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
        let result = sqlx::query_file!("queries/folders/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

impl FolderRepository for PostgresFolderRepository {}

#[derive(Debug, PartialEq, sqlx::Type)]
pub enum FolderType {
    Feeds,
    Collections,
}

impl From<colette_core::folder::FolderType> for FolderType {
    fn from(value: colette_core::folder::FolderType) -> Self {
        match value {
            colette_core::folder::FolderType::Feeds => FolderType::Feeds,
            colette_core::folder::FolderType::Collections => FolderType::Collections,
        }
    }
}

impl From<FolderType> for colette_core::folder::FolderType {
    fn from(value: FolderType) -> Self {
        match value {
            FolderType::Feeds => colette_core::folder::FolderType::Feeds,
            FolderType::Collections => colette_core::folder::FolderType::Collections,
        }
    }
}
