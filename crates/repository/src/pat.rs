use chrono::{DateTime, Utc};
use colette_core::{
    auth::{LookupHash, PatByLookupHash, PatFindParams, PatRepository, PersonalAccessToken},
    common::RepositoryError,
};
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
    async fn find(
        &self,
        params: PatFindParams,
    ) -> Result<Vec<PersonalAccessToken>, RepositoryError> {
        let pats = sqlx::query_file_as!(
            PersonalAccessTokenRow,
            "queries/pats/find.sql",
            params.user_id.as_inner(),
            params.id.map(|e| e.as_inner()),
            params.cursor,
            params.limit.map(|e| e as i64)
        )
        .map(Into::into)
        .fetch_all(&self.pool)
        .await?;

        Ok(pats)
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
}

#[derive(serde::Deserialize)]
pub(crate) struct PersonalAccessTokenRow {
    id: Uuid,
    lookup_hash: String,
    verification_hash: String,
    title: String,
    preview: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<PersonalAccessTokenRow> for PersonalAccessToken {
    fn from(value: PersonalAccessTokenRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.lookup_hash,
            value.verification_hash,
            value.title,
            value.preview,
            value.created_at,
            value.updated_at,
        )
    }
}

#[derive(serde::Deserialize)]
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
