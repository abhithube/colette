use chrono::{DateTime, Utc};
use colette_core::{
    Collection,
    bookmark::BookmarkFilter,
    collection::{
        CollectionById, CollectionFindParams, CollectionInsertParams, CollectionRepository,
        CollectionUpdateParams, Error,
    },
};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresCollectionRepository {
    pool: PgPool,
}

impl PostgresCollectionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl CollectionRepository for PostgresCollectionRepository {
    async fn find(&self, params: CollectionFindParams) -> Result<Vec<Collection>, Error> {
        let collections = sqlx::query_file_as!(
            CollectionRow,
            "queries/collections/find.sql",
            params.id,
            params.user_id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(collections)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<CollectionById>, Error> {
        let collection =
            sqlx::query_file_as!(CollectionByIdRow, "queries/collections/find_by_id.sql", id)
                .map(Into::into)
                .fetch_optional(&self.pool)
                .await?;

        Ok(collection)
    }

    async fn insert(&self, params: CollectionInsertParams) -> Result<Uuid, Error> {
        let id = sqlx::query_file_scalar!(
            "queries/collections/insert.sql",
            params.title,
            Json(params.filter) as Json<BookmarkFilter>,
            params.user_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::Database(e) if e.is_unique_violation() => Error::Conflict(params.title),
            _ => Error::Sqlx(e),
        })?;

        Ok(id)
    }

    async fn update(&self, params: CollectionUpdateParams) -> Result<(), Error> {
        sqlx::query_file!(
            "queries/collections/update.sql",
            params.id,
            params.title,
            params.filter.map(Json) as Option<Json<BookmarkFilter>>
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: Uuid) -> Result<(), Error> {
        sqlx::query_file!("queries/collections/delete_by_id.sql", id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

struct CollectionRow {
    id: Uuid,
    title: String,
    filter_json: Json<BookmarkFilter>,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<CollectionRow> for Collection {
    fn from(value: CollectionRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            filter: value.filter_json.0,
            user_id: value.user_id,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

struct CollectionByIdRow {
    id: Uuid,
    user_id: Uuid,
}

impl From<CollectionByIdRow> for CollectionById {
    fn from(value: CollectionByIdRow) -> Self {
        Self {
            id: value.id,
            user_id: value.user_id,
        }
    }
}
