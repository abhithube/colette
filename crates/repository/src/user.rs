use chrono::{DateTime, Utc};
use colette_core::{
    User,
    auth::{OtpCode, SocialAccount, UserId, UserRepository},
    common::RepositoryError,
};
use email_address::EmailAddress;
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::{DbUrl, pat::PersonalAccessTokenRow};

#[derive(Debug, Clone)]
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_file_as!(
            UserRow,
            "queries/users/find_by_unique.sql",
            id.as_inner(),
            Option::<String>::None,
            Option::<String>::None,
            Option::<String>::None,
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_email(&self, email: EmailAddress) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_file_as!(
            UserRow,
            "queries/users/find_by_unique.sql",
            Option::<Uuid>::None,
            email.as_str(),
            Option::<String>::None,
            Option::<String>::None,
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn find_by_provider_and_sub(
        &self,
        provider: String,
        sub: String,
    ) -> Result<Option<User>, RepositoryError> {
        let user = sqlx::query_file_as!(
            UserRow,
            "queries/users/find_by_unique.sql",
            Option::<Uuid>::None,
            Option::<String>::None,
            provider,
            sub
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(user)
    }

    async fn save(&self, data: &User) -> Result<(), RepositoryError> {
        let mut oc_codes = Vec::<String>::new();
        let mut oc_expired_ats = Vec::<DateTime<Utc>>::new();
        let mut oc_used_ats = Vec::<Option<DateTime<Utc>>>::new();
        let mut oc_created_ats = Vec::<DateTime<Utc>>::new();
        let mut oc_updated_ats = Vec::<DateTime<Utc>>::new();

        for oc in data.otp_codes() {
            oc_codes.push(oc.code().to_owned());
            oc_expired_ats.push(oc.expires_at());
            oc_used_ats.push(oc.used_at());
            oc_created_ats.push(oc.created_at());
            oc_updated_ats.push(oc.updated_at());
        }

        let mut sa_providers = Vec::<String>::new();
        let mut sa_subs = Vec::<String>::new();
        let mut sa_created_ats = Vec::<DateTime<Utc>>::new();
        let mut sa_updated_ats = Vec::<DateTime<Utc>>::new();

        for sa in data.social_accounts() {
            sa_providers.push(sa.provider().to_string());
            sa_subs.push(sa.sub().to_string());
            sa_created_ats.push(sa.created_at());
            sa_updated_ats.push(sa.updated_at());
        }

        let mut pat_ids = Vec::<Uuid>::new();
        let mut pat_lookup_hashes = Vec::<String>::new();
        let mut pat_verification_hashes = Vec::<String>::new();
        let mut pat_titles = Vec::<String>::new();
        let mut pat_previews = Vec::<String>::new();
        let mut pat_created_ats = Vec::<DateTime<Utc>>::new();
        let mut pat_updated_ats = Vec::<DateTime<Utc>>::new();

        for pat in data.personal_access_tokens() {
            pat_ids.push(pat.id().as_inner());
            pat_lookup_hashes.push(pat.lookup_hash().as_inner().to_owned());
            pat_verification_hashes.push(pat.verification_hash().as_inner().to_owned());
            pat_titles.push(pat.title().as_inner().to_owned());
            pat_previews.push(pat.preview().as_inner().to_owned());
            pat_created_ats.push(pat.created_at());
            pat_updated_ats.push(pat.updated_at());
        }

        sqlx::query_file!(
            "queries/users/upsert.sql",
            data.id().as_inner(),
            data.email().as_str(),
            data.verified(),
            data.display_name().map(|e| e.as_inner()),
            data.image_url().cloned().map(Into::into) as Option<DbUrl>,
            data.created_at(),
            data.updated_at(),
            &oc_codes,
            &oc_expired_ats,
            &oc_used_ats as &[Option<DateTime<Utc>>],
            &oc_created_ats,
            &oc_updated_ats,
            &sa_providers,
            &sa_subs,
            &sa_created_ats,
            &sa_updated_ats,
            &pat_ids,
            &pat_lookup_hashes,
            &pat_verification_hashes,
            &pat_titles,
            &pat_previews,
            &pat_created_ats,
            &pat_updated_ats
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct UserRow {
    id: Uuid,
    email: String,
    verified: bool,
    display_name: Option<String>,
    image_url: Option<DbUrl>,
    social_accounts: Json<Vec<SocialAccountRow>>,
    personal_access_tokens: Json<Vec<PersonalAccessTokenRow>>,
    otp_codes: Json<Vec<OtpCodeRow>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(value: UserRow) -> Self {
        Self::from_unchecked(
            value.id,
            value.email,
            value.verified,
            value.display_name,
            value.image_url.map(Into::into),
            value.otp_codes.0.into_iter().map(Into::into).collect(),
            value
                .social_accounts
                .0
                .into_iter()
                .map(Into::into)
                .collect(),
            value
                .personal_access_tokens
                .0
                .into_iter()
                .map(Into::into)
                .collect(),
            value.created_at,
            value.updated_at,
        )
    }
}

#[derive(serde::Deserialize)]
struct OtpCodeRow {
    code: String,
    expires_at: DateTime<Utc>,
    used_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<OtpCodeRow> for OtpCode {
    fn from(value: OtpCodeRow) -> Self {
        Self::from_unchecked(
            value.code,
            value.expires_at,
            value.used_at,
            value.created_at,
            value.updated_at,
        )
    }
}

#[derive(serde::Deserialize)]
struct SocialAccountRow {
    provider: String,
    sub: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl From<SocialAccountRow> for SocialAccount {
    fn from(value: SocialAccountRow) -> Self {
        Self::from_unchecked(
            value.provider,
            value.sub,
            value.created_at,
            value.updated_at,
        )
    }
}
