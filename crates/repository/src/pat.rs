use chrono::{DateTime, Utc};
use colette_authentication::{
    LookupHash, PatByLookupHash, PatId, PatRepository, PersonalAccessToken, UserId,
};
use colette_common::RepositoryError;
use colette_handler::{PatQueryParams, PatQueryRepository, PersonalAccessTokenDto};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresPatRepository {
    pool: PgPool,
}

impl PostgresPatRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PatRepository for PostgresPatRepository {
    async fn find_by_id(
        &self,
        id: PatId,
        user_id: UserId,
    ) -> Result<Option<PersonalAccessToken>, RepositoryError> {
        let pat = sqlx::query_file_as!(
            PatByIdRow,
            "queries/pats/find_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(pat)
    }

    async fn find_by_lookup_hash(
        &self,
        lookup_hash: &LookupHash,
    ) -> Result<Option<PatByLookupHash>, RepositoryError> {
        let pat = sqlx::query_file_as!(
            PatByLookupHashRow,
            "queries/pats/find_by_lookup_hash.sql",
            lookup_hash.as_inner(),
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(pat)
    }

    async fn count(&self, user_id: UserId) -> Result<u8, RepositoryError> {
        let count = sqlx::query_file_scalar!("queries/pats/count.sql", user_id.as_inner())
            .fetch_one(&self.pool)
            .await?;

        Ok(count.unwrap_or(0) as u8)
    }

    async fn save(&self, data: &PersonalAccessToken) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/pats/upsert.sql",
            data.id().as_inner(),
            data.lookup_hash().as_inner(),
            data.verification_hash().as_inner(),
            data.title().as_inner(),
            data.preview().as_inner(),
            data.user_id().as_inner(),
            data.created_at(),
            data.updated_at(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_by_id(&self, id: PatId, user_id: UserId) -> Result<(), RepositoryError> {
        sqlx::query_file!(
            "queries/pats/delete_by_id.sql",
            id.as_inner(),
            user_id.as_inner()
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => RepositoryError::NotFound,
            _ => RepositoryError::Unknown(e),
        })?;

        Ok(())
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct PatByIdRow {
    id: Uuid,
    lookup_hash: String,
    verification_hash: String,
    title: String,
    preview: String,
    user_id: Uuid,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PatByIdRow> for PersonalAccessToken {
    fn from(value: PatByIdRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.lookup_hash,
            value.verification_hash,
            value.title,
            value.preview,
            value.user_id,
            value.created_at,
            value.updated_at,
        )
    }
}

pub(crate) struct PatByLookupHashRow {
    id: Uuid,
    verification_hash: String,
    user_id: Uuid,
}

impl From<PatByLookupHashRow> for PatByLookupHash {
    fn from(value: PatByLookupHashRow) -> Self {
        Self::from_unchecked(value.id, value.verification_hash, value.user_id)
    }
}

#[async_trait::async_trait]
impl PatQueryRepository for PostgresPatRepository {
    async fn query(
        &self,
        params: PatQueryParams,
    ) -> Result<Vec<PersonalAccessTokenDto>, RepositoryError> {
        let pats = sqlx::query_file_as!(
            PersonalAccessTokenRow,
            "queries/pats/find.sql",
            params.user_id,
            params.id,
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(pats)
    }
}

#[derive(serde::Deserialize)]
pub(crate) struct PersonalAccessTokenRow {
    id: Uuid,
    title: String,
    preview: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PersonalAccessTokenRow> for PersonalAccessTokenDto {
    fn from(value: PersonalAccessTokenRow) -> Self {
        Self {
            id: value.id,
            title: value.title,
            preview: value.preview,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}
