use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use sea_orm::{
    prelude::Uuid,
    sqlx::{self, PgExecutor},
    DatabaseConnection,
};

pub struct TagSqlRepository {
    pub(crate) db: DatabaseConnection,
}

impl TagSqlRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Findable for TagSqlRepository {
    type Params = IdParams;
    type Output = Result<Tag, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        find_by_id(self.db.get_postgres_connection_pool(), params).await
    }
}

#[async_trait::async_trait]
impl Creatable for TagSqlRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = colette_postgres::tag::insert(
            self.db.get_postgres_connection_pool(),
            Uuid::new_v4(),
            data.title.clone(),
            data.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
            _ => Error::Unknown(e.into()),
        })?;

        let tag = find_by_id(&mut *tx, IdParams::new(id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Updatable for TagSqlRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut tx = self
            .db
            .get_postgres_connection_pool()
            .begin()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        colette_postgres::tag::update(&mut *tx, params.id, params.profile_id, data.title)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })?;

        let tag = find_by_id(&mut *tx, params)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Deletable for TagSqlRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        colette_postgres::tag::delete_by_id(
            self.db.get_postgres_connection_pool(),
            params.id,
            params.profile_id,
        )
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => Error::NotFound(params.id),
            _ => Error::Unknown(e.into()),
        })
    }
}

#[async_trait::async_trait]
impl TagRepository for TagSqlRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Vec<Tag>, Error> {
        find(
            self.db.get_postgres_connection_pool(),
            None,
            profile_id,
            limit,
            cursor,
            filters,
        )
        .await
    }
}

pub(crate) async fn find(
    executor: impl PgExecutor<'_>,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<Tag>, Error> {
    let tags = colette_postgres::tag::select(executor, id, profile_id, limit, cursor, filters)
        .await
        .map(|e| e.into_iter().map(Tag::from).collect::<Vec<_>>())
        .map_err(|e| Error::Unknown(e.into()))?;

    Ok(tags)
}

async fn find_by_id(executor: impl PgExecutor<'_>, params: IdParams) -> Result<Tag, Error> {
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
