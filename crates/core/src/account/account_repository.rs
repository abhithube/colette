use uuid::Uuid;

use super::{Account, Error};

#[async_trait::async_trait]
pub trait AccountRepository: Send + Sync + 'static {
    async fn find_account(&self, params: AccountFindParams) -> Result<Account, Error>;

    async fn create_account(&self, data: AccountCreateData) -> Result<Uuid, Error>;
}

#[derive(Debug, Clone, Default)]
pub struct AccountFindParams {
    pub provider_id: String,
    pub account_id: String,
}

#[derive(Debug, Clone, Default)]
pub struct AccountCreateData {
    pub email: String,
    pub display_name: Option<String>,
    pub provider_id: String,
    pub account_id: String,
    pub password_hash: Option<String>,
}
