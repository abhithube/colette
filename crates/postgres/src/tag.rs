use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Cursor, Error, TagCreateData, TagFindManyFilters, TagRepository, TagUpdateData},
    Tag,
};
use deadpool_postgres::{GenericClient, Pool};
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{error::SqlState, Row};
use uuid::Uuid;

pub struct PostgresTagRepository {
    pub(crate) pool: Pool,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<Tag, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find_by_id(&client, params).await
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresTagRepository {
    type Data = TagCreateData;
    type Output = Result<Tag, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let id = Uuid::new_v4();

        {
            let (sql, values) = colette_sql::tag::insert(id, data.title.clone(), data.profile_id)
                .build_postgres(PostgresQueryBuilder);

            let stmt = tx
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            tx.execute(&stmt, &values.as_params())
                .await
                .map_err(|e| match e.code() {
                    Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title),
                    _ => Error::Unknown(e.into()),
                })?;
        };

        let tag = find_by_id(&tx, IdParams::new(id, data.profile_id))
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<Tag, Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        let mut client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let tx = client
            .transaction()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        if data.title.is_some() {
            let count = {
                let (sql, values) =
                    colette_sql::tag::update(params.id, params.profile_id, data.title)
                        .build_postgres(PostgresQueryBuilder);

                let stmt = tx
                    .prepare_cached(&sql)
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?;

                tx.execute(&stmt, &values.as_params())
                    .await
                    .map_err(|e| Error::Unknown(e.into()))?
            };
            if count == 0 {
                return Err(Error::NotFound(params.id));
            }
        }

        let tag = find_by_id(&tx, params)
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        tx.commit().await.map_err(|e| Error::Unknown(e.into()))?;

        Ok(tag)
    }
}

#[async_trait::async_trait]
impl Deletable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        let count = {
            let (sql, values) = colette_sql::tag::delete_by_id(params.id, params.profile_id)
                .build_postgres(PostgresQueryBuilder);

            let stmt = client
                .prepare_cached(&sql)
                .await
                .map_err(|e| Error::Unknown(e.into()))?;

            client
                .execute(&stmt, &values.as_params())
                .await
                .map_err(|e| Error::Unknown(e.into()))?
        };
        if count == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl TagRepository for PostgresTagRepository {
    async fn list(
        &self,
        profile_id: Uuid,
        limit: Option<u64>,
        cursor: Option<Cursor>,
        filters: Option<TagFindManyFilters>,
    ) -> Result<Vec<Tag>, Error> {
        let client = self
            .pool
            .get()
            .await
            .map_err(|e| Error::Unknown(e.into()))?;

        find(&client, None, profile_id, limit, cursor, filters).await
    }
}

#[derive(Debug, Clone)]
struct TagSelect(Tag);

impl From<&Row> for TagSelect {
    fn from(value: &Row) -> Self {
        Self(Tag {
            id: value.get("id"),
            title: value.get("title"),
            bookmark_count: Some(value.get("bookmark_count")),
            feed_count: Some(value.get("feed_count")),
        })
    }
}

pub(crate) async fn find<C: GenericClient>(
    client: &C,
    id: Option<Uuid>,
    profile_id: Uuid,
    limit: Option<u64>,
    cursor: Option<Cursor>,
    filters: Option<TagFindManyFilters>,
) -> Result<Vec<Tag>, Error> {
    let (sql, values) = colette_sql::tag::select(id, profile_id, limit, cursor, filters)
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
                .map(|e| TagSelect::from(&e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
}

async fn find_by_id<C: GenericClient>(client: &C, params: IdParams) -> Result<Tag, Error> {
    let mut tags = find(client, Some(params.id), params.profile_id, None, None, None).await?;
    if tags.is_empty() {
        return Err(Error::NotFound(params.id));
    }

    Ok(tags.swap_remove(0))
}
