use uuid::Uuid;

use super::{Error, User};

#[async_trait::async_trait]
pub trait UserRepository: Send + Sync + 'static {
    async fn query(&self, params: UserParams) -> Result<Vec<User>, Error>;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Error> {
        Ok(self
            .query(UserParams { id: Some(id) })
            .await?
            .into_iter()
            .next())
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserParams {
    pub id: Option<Uuid>,
}
