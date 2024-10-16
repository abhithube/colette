use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use deadpool_sqlite::Pool;
use rusqlite::{Connection, Row};
use sea_query::SqliteQueryBuilder;
use sea_query_rusqlite::RusqliteBinder;
use uuid::Uuid;

pub struct SqliteTagRepository {
    pool: Pool,
}

impl SqliteTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteTagRepository {
    type Params = IdParams;
    type Output = Result<Tag, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find_by_id(conn, params.id, params.profile_id))
            .await
            .unwrap()
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteTagRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let title = data.title.clone();

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            let id = Uuid::new_v4();

            {
                let (sql, values) =
                    colette_sql::tag::insert(id, data.title.clone(), data.profile_id)
                        .build_rusqlite(SqliteQueryBuilder);

                tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
            }

            let tag = find_by_id(&tx, id, data.profile_id)?;

            tx.commit()?;

            Ok(tag)
        })
        .await
        .unwrap()
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(e, _)
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Error::Conflict(title)
            }
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let tx = conn.transaction()?;

            if data.title.is_some() {
                let (sql, values) =
                    colette_sql::tag::update(params.id, params.profile_id, data.title)
                        .build_rusqlite(SqliteQueryBuilder);

                let count = tx.prepare_cached(&sql)?.execute(&*values.as_params())?;
                if count == 0 {
                    return Err(rusqlite::Error::QueryReturnedNoRows);
                }
            }

            let tag = find_by_id(&tx, params.id, params.profile_id)?;

            tx.commit()?;

            Ok(tag)
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
impl Deletable for SqliteTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| {
            let (sql, values) = colette_sql::tag::delete_by_id(params.id, params.profile_id)
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
impl TagRepository for SqliteTagRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Vec<Tag>, Error> {
        let conn = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        conn.interact(move |conn| find(conn, None, profile_id, limit, cursor, filters))
            .await
            .unwrap()
            .map_err(|e| Error::Unknown(e.into()))
    }
}

#[derive(Debug, Clone)]
struct TagSelect(Tag);

impl TryFrom<&Row<'_>> for TagSelect {
    type Error = rusqlite::Error;

    fn try_from(value: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Self(Tag {
            id: value.get("id")?,
            title: value.get("title")?,
            bookmark_count: Some(value.get("bookmark_count")?),
            feed_count: Some(value.get("feed_count")?),
        }))
    }
}

pub(crate) fn find(
    conn: &Connection,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> rusqlite::Result<Vec<Tag>> {
    let (sql, values) = colette_sql::tag::select(id, profile_id, limit, cursor, filters)
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare_cached(&sql)?;
    let mut rows = stmt.query(&*values.as_params())?;

    let mut tags: Vec<Tag> = Vec::new();
    while let Some(row) = rows.next()? {
        tags.push(TagSelect::try_from(row).map(|e| e.0)?);
    }

    Ok(tags)
}

fn find_by_id(conn: &Connection, id: Uuid, profile_id: Uuid) -> rusqlite::Result<Tag> {
    let mut tags = find(conn, Some(id), profile_id, None, None, None)?;
    if tags.is_empty() {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(tags.swap_remove(0))
}
