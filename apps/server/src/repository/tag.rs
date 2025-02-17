use colette_core::{
    Tag,
    common::{Creatable, Deletable, Findable, IdParams, Updatable},
    tag::{Error, TagCreateData, TagFindParams, TagRepository, TagUpdateData},
};
use sqlx::{Pool, Postgres};
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
        let tags = sqlx::query_file_as!(
            Tag,
            "queries/tags/select.sql",
            params.user_id,
            params.id.is_none(),
            params.id,
            params.cursor.is_none(),
            params.cursor.map(|e| e.title),
            params.limit
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(tags)
    }
}

#[async_trait::async_trait]
impl Creatable for PostgresTagRepository {
    type Data = TagCreateData;
    type Output = Result<Uuid, Error>;

    async fn create(&self, data: Self::Data) -> Self::Output {
        sqlx::query_file_scalar!("queries/tags/insert.sql", data.title, data.user_id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(data.title),
                _ => Error::Database(e),
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
            sqlx::query_file!(
                "queries/tags/update.sql",
                params.id,
                params.user_id,
                data.title.is_some(),
                data.title.map(String::from)
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
impl Deletable for PostgresTagRepository {
    type Params = IdParams;
    type Output = Result<(), Error>;

    async fn delete(&self, params: Self::Params) -> Self::Output {
        let result = sqlx::query_file!("queries/tags/delete.sql", params.id, params.user_id)
            .execute(&self.pool)
            .await?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound(params.id));
        }

        Ok(())
    }
}

impl TagRepository for PostgresTagRepository {}
