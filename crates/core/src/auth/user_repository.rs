use uuid::Uuid;

use super::{Error, User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn query(&self, params: UserParams) -> Result<Vec<User>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        Ok(self
            .query(UserParams {
                id: Some(id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn find_by_external_id(&self, external_id: String) -> Result<Option<User>, Error> {
        Ok(self
            .query(UserParams {
                external_id: Some(external_id),
                ..Default::default()
            })
            .await?
            .into_iter()
            .next())
    }

    async fn save(&self, data: &User) -> Result<(), Error>;
}

#[derive(Debug, Clone, Default)]
pub struct UserParams {
    pub id: Option<Uuid>,
    pub external_id: Option<String>,
}
