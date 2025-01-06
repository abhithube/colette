use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    folder::{Error, FolderCreateData, FolderFindParams, FolderRepository, FolderUpdateData},
    Folder,
};
use deadpool_sqlite::{
    rusqlite::{self, Row},
    Pool,
};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteFolderRepository {
    pool: Pool,
}

impl SqliteFolderRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteFolderRepository {
    type Params = FolderFindParams;
    type Output = Result<Vec<Folder>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::folder::select(
                params.id,
                params.user_id,
                params.parent_id,
                params.limit,
                params.cursor,
            )
            .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut folders: Vec<Folder> = Vec::new();
            while let Some(row) = rows.next()? {
                folders.push(FolderSelect::try_from(row).map(|e| e.0)?);
            }

            Ok(folders)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteFolderRepository {
    type Data = FolderCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::folder::insert(
                Some(Uuid::new_v4()),
                data.title,
                data.parent_id,
                data.user_id,
            )
            .build_rusqlite(SqliteQueryBuilder);

            conn.prepare_cached(&sql)?
                .query_row(&*values.as_params(), |row| row.get::<_, Uuid>("id"))
        })
        .await
        .unwrap()
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteFolderRepository {
    type Params = IdParams;
    type Data = FolderUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_none() {
            return Ok(());
        }

        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) =
                crate::folder::update(params.id, params.user_id, data.title, data.parent_id)
                    .build_rusqlite(SqliteQueryBuilder);

            let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
            if count == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteFolderRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::folder::delete_by_id(params.id, params.user_id)
                .build_rusqlite(SqliteQueryBuilder);

            let count = conn.prepare_cached(&sql)?.execute(&*values.as_params())?;
            if count == 0 {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            }

            Ok(())
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

impl FolderRepository for SqliteFolderRepository {}

#[derive(Debug, Clone)]
struct FolderSelect(Folder);

impl TryFrom<&Row<'_>> for FolderSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Folder {
            id: value.get("id")?,
            title: value.get("title")?,
            parent_id: value.get("parent_id")?,
        }))
    }
}
