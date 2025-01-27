use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
    Folder,
};
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
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
        crate::folder::select(
            &self.pool,
            params.id,
            params.user_id,
            params.parent_id,
            params.limit,
            params.cursor,
        )
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
        crate::folder::insert(&self.pool, data.title.clone(), data.parent_id, data.user_id)
            .await
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresFolderRepository {
    type Params = IdParams;
    type Data = FolderUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            crate::folder::update(
                &self.pool,
                params.id,
                params.user_id,
                data.title,
                data.parent_id,
            )
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
impl Deletable for PostgresFolderRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        crate::folder::delete(&self.pool, params.id, params.user_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        Ok(())
    }
}

impl FolderRepository for PostgresFolderRepository {}

#[derive(Debug, Clone)]
pub(crate) struct FolderSelect(pub(crate) Folder);

impl From<PgRow> for FolderSelect {
    fn from(value: PgRow) -> Self {
        Self(Folder {
            id: value.get("id"),
            title: value.get("title"),
            parent_id: value.get("parent_id"),
        })
    }
}
