use uuid::Uuid;

use crate::RepositoryError;

#[async_trait::async_trait]
pub trait AccountRepository: Send + Sync + 'static {
    async fn find_by_sub_and_provider(
        &self,
        sub: String,
        provider: String,
    ) -> Result<Option<AccountBySubAndProvider>, RepositoryError>;

    async fn insert(&self, params: AccountInsertParams) -> Result<Uuid, RepositoryError>;

    async fn update(&self, params: AccountUpdateParams) -> Result<(), RepositoryError>;
}

#[derive(Debug, Clone)]
pub struct AccountBySubAndProvider {
    pub id: Uuid,
    pub password_hash: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AccountInsertParams {
    pub sub: String,
    pub provider: String,
    pub password_hash: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct AccountUpdateParams {
    pub id: Uuid,
    pub password_hash: Option<Option<String>>,
}
