use uuid::Uuid;

use super::{Account, Error};

#[async_trait::async_trait]
pub trait AccountRepository: Send + Sync + 'static {
    async fn query(&self, params: AccountParams) -> Result<Vec<Account>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Account>, Error> {
        Ok(self
            .query(AccountParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn find_by_sub_and_provider(
        &self,
        sub: String,
        provider: String,
    ) -> Result<Option<Account>, Error> {
        Ok(self
            .query(AccountParams {
                sub: Some(sub),
                provider: Some(provider),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &Account) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct AccountParams {
    pub id: Option<Uuid>,
    pub sub: Option<String>,
    pub provider: Option<String>,
}
