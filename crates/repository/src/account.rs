use colette_core::{
    RepositoryError,
    account::{
        AccountBySubAndProvider, AccountInsertParams, AccountRepository, AccountUpdateParams,
    },
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PostgresAccountRepository {
    pool: PgPool,
}

impl PostgresAccountRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl AccountRepository for PostgresAccountRepository {
    async fn find_by_sub_and_provider(
        &self,
        sub: String,
        provider: String,
    ) -> Result<Option<AccountBySubAndProvider>, RepositoryError> {
        let account = sqlx::query_file_as!(
            AccountBySubAndProviderRow,
            "queries/accounts/find_by_sub_and_provider.sql",
            sub,
            provider
        )
        .map(Into::into)
        .fetch_optional(&self.pool)
        .await?;

        Ok(account)
    }

    async fn insert(&self, params: AccountInsertParams) -> Result<Uuid, RepositoryError> {
        let id = sqlx::query_file_scalar!(
            "queries/accounts/insert.sql",
            params.sub,
            params.provider,
            params.password_hash,
            params.user_id.as_inner()
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    async fn update(&self, params: AccountUpdateParams) -> Result<(), RepositoryError> {
        let (has_password_hash, password_hash) = if let Some(password_hash) = params.password_hash {
            (true, password_hash)
        } else {
            (false, None)
        };

        sqlx::query_file!(
            "queries/accounts/update.sql",
            params.id.as_inner(),
            has_password_hash,
            password_hash
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

struct AccountBySubAndProviderRow {
    id: Uuid,
    password_hash: Option<String>,
    user_id: Uuid,
}

impl From<AccountBySubAndProviderRow> for AccountBySubAndProvider {
    fn from(value: AccountBySubAndProviderRow) -> Self {
        Self {
            id: value.id,
            password_hash: value.password_hash,
            user_id: value.user_id.into(),
        }
    }
}
