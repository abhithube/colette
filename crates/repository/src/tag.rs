use colette_core::{
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
    Tag,
};
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: Pool<Postgres>,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Findable for PostgresTagRepository {
    type Params = TagFindParams;
    type Output = Result<Vec<Tag>, Error>;

    async fn find(&self, params: Self::Params) -> Self::Output {
        crate::query::tag::select(
            &self.pool,
            params.id,
            params.user_id,
            params.limit,
            params.cursor,
            params.tag_type,
        )
        .await
        .map(|e| {
            e.into_iter()
                .map(|e| TagSelect::from(e).0)
                .collect::<Vec<_>>()
        })
        .map_err(|e| Error::Unknown(e.into()))
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresTagRepository {
    type Data = TagCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        crate::query::tag::insert(&self.pool, data.title.clone(), data.user_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
                _ => Error::Unknown(e.into()),
            })
    }
}

#[async_trait::async_trait]
impl Updatable for PostgresTagRepository {
    type Params = IdParams;
    type Data = TagUpdateData;
    type Output = Result<(), Error>;

    async fn update(&self, params: Self::Params, data: Self::Data) -> Self::Output {
        if data.title.is_some() {
            crate::query::tag::update(&self.pool, params.id, params.user_id, data.title)
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
impl Deletable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        crate::query::tag::delete(&self.pool, params.id, params.user_id)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => Error::NotFound(params.id),
                _ => Error::Unknown(e.into()),
            })
    }
}

impl TagRepository for PostgresTagRepository {}

#[derive(Debug, Clone)]
struct TagSelect(Tag);

impl From<PgRow> for TagSelect {
    fn from(value: PgRow) -> Self {
        Self(Tag {
            id: value.get("id"),
            title: value.get("title"),
            bookmark_count: Some(value.get("bookmark_count")),
            feed_count: Some(value.get("feed_count")),
        })
    }
}
