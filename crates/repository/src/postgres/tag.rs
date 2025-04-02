use colette_core::{
    Tag,
    tag::{Error, TagParams, TagRepository},
};
use colette_query::{
    IntoDelete, IntoInsert, IntoSelect,
    tag::{TagDelete, TagInsert},
};
use deadpool_postgres::Pool;
use sea_query::PostgresQueryBuilder;
use sea_query_postgres::PostgresBinder;
use tokio_postgres::{Row, error::SqlState};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresTagRepository {
    pool: Pool,
}

impl PostgresTagRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl TagRepository for PostgresTagRepository {
    async fn query(&self, params: TagParams) -> Result<Vec<Tag>, Error> {
        let client = self.pool.get().await?;

        let (sql, values) = params.into_select().build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        let rows = client.query(&stmt, &values.as_params()).await?;

        Ok(rows.iter().map(|e| TagRow(e).into()).collect())
    }

    async fn save(&self, data: &Tag) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = TagInsert {
            id: data.id,
            title: &data.title,
            user_id: &data.user_id,
            created_at: data.created_at,
            updated_at: data.updated_at,
            upsert: false,
        }
        .into_insert()
        .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client
            .execute(&stmt, &values.as_params())
            .await
            .map_err(|e| match e.code() {
                Some(&SqlState::UNIQUE_VIOLATION) => Error::Conflict(data.title.clone()),
                _ => Error::Database(e),
            })?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        let client = self.pool.get().await?;

        let (sql, values) = TagDelete { id }
            .into_delete()
            .build_postgres(PostgresQueryBuilder);

        let stmt = client.prepare_cached(&sql).await?;
        client.execute(&stmt, &values.as_params()).await?;

        Ok(())
    }
}

struct TagRow<'a>(&'a Row);

impl From<TagRow<'_>> for Tag {
    fn from(TagRow(value): TagRow<'_>) -> Self {
        Self {
            id: value.get("id"),
            title: value.get("title"),
            user_id: value.get("user_id"),
            created_at: value.get("created_at"),
            updated_at: value.get("updated_at"),
            feed_count: value.try_get("feed_count").ok(),
            bookmark_count: value.try_get("bookmark_count").ok(),
        }
    }
}
