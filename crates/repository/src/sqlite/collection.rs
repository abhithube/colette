use colette_core::{
    collection::{
        CollectionCreateData, CollectionFindParams, CollectionRepository, CollectionUpdateData,
        Error,
    },
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    Collection,
};
use deadpool_sqlite::{
    rusqlite::{self, Row},
    Pool,
};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteCollectionRepository {
    pool: Pool,
}

impl SqliteCollectionRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteCollectionRepository {
    type Params = CollectionFindParams;
    type Output = Result<Vec<Collection>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) =
                crate::collection::select(params.id, params.user_id, params.limit, params.cursor)
                    .build_rusqlite(SqliteQueryBuilder);

            let mut stmt = conn.prepare_cached(&sql)?;
            let mut rows = stmt.query(&*values.as_params())?;

            let mut collections: Vec<Collection> = Vec::new();
            while let Some(row) = rows.next()? {
                collections.push(CollectionSelect::try_from(row).map(|e| e.0)?);
            }

            Ok(collections)
        })
        .await
        .unwrap()
        .map_err(|e: rusqlite::Error| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteCollectionRepository {
    type Data = CollectionCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) =
                crate::collection::insert(Some(Uuid::new_v4()), data.title, data.user_id)
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
impl Updatable for SqliteCollectionRepository {
    type Params = IdParams;
    type Data = CollectionUpdateData;
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
            let (sql, values) = crate::collection::update(params.id, params.user_id, data.title)
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
impl Deletable for SqliteCollectionRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = crate::collection::delete_by_id(params.id, params.user_id)
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

impl CollectionRepository for SqliteCollectionRepository {}

#[derive(Debug, Clone)]
struct CollectionSelect(Collection);

impl TryFrom<&Row<'_>> for CollectionSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Collection {
            id: value.get("id")?,
            title: value.get("title")?,
        }))
    }
}
