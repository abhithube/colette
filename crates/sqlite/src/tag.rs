use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::{SqliteExecutor, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SqliteTagRepository {
    pool: SqlitePool,
}

impl SqliteTagRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for SqliteTagRepository {
    type Params = IdParams;
    type Output = Result<Tag, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(&self.pool, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for SqliteTagRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = {
            let (sql, values) =
                colette_sql::tag::insert(Some(Uuid::new_v4()), data.title.clone(), data.profile_id)
                    .build_sqlx(SqliteQueryBuilder);

            sqlx::query_scalar_with::<_, Uuid, _>(&sql, values)
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| match e {
                    sqlx::Error::Database(e) if e.is_unique_violation() => {
                        Error::Conflict(data.title)
                    }
                    _ => Error::Unknown(e.into()),
                })?
        };

        let tag = find_by_id(&mut *tx, IdParams::new(id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Updatable for SqliteTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let count = {
                let (sql, values) =
                    colette_sql::tag::update(params.id, params.profile_id, data.title)
                        .build_sqlx(SqliteQueryBuilder);

                sqlx::query_with(&sql, values)
                    .execute(&mut *tx)
                    .await
                    .map(|e| e.rows_affected())
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let tag = find_by_id(&mut *tx, params)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Deletable for SqliteTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let count = {
            let (sql, values) = colette_sql::tag::delete_by_id(params.id, params.profile_id)
                .build_sqlx(SqliteQueryBuilder);

            sqlx::query_with(&sql, values)
                .execute(&self.pool)
                .await
                .map(|e| e.rows_affected())
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
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
        find(&self.pool, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
struct TagSelect {
    id: Uuid,
    title: String,
    bookmark_count: i64,
    feed_count: i64,
}

impl From<TagSelect> for colette_core::Tag {
    fn from(value: TagSelect) -> Self {
        Self {
            id: value.id,
            title: value.title,
            bookmark_count: Some(value.bookmark_count),
            feed_count: Some(value.feed_count),
        }
    }
}

pub(crate) async fn find(
    executor: impl SqliteExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<Tag>, Error> {
    let (sql, values) = colette_sql::tag::select(id, profile_id, limit, cursor, filters)
        .build_sqlx(SqliteQueryBuilder);

    sqlx::query_as_with::<_, TagSelect, _>(&sql, values)
        .fetch_all(executor)
        .await
        .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id(executor: impl SqliteExecutor<'_>, params: IdParams) -> Result<Tag, Error> {
    let mut tags = find(
        executor,
        Some(params.id),
        params.profile_id,
        None,
        None,
        None,
    )
    .await?;
    if tags.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(tags.swap_remove(0))
}
