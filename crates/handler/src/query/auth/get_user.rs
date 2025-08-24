use colette_authentication::{User, UserId, UserRepository};
use colette_common::RepositoryError;

use crate::Handler;

#[derive(Debug, Clone)]
pub struct GetUserQuery {
    pub id: UserId,
}

pub struct GetUserHandler<UR: UserRepository> {
    user_repository: UR,
}

impl<UR: UserRepository> GetUserHandler<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository> Handler<GetUserQuery> for GetUserHandler<UR> {
    type Response = User;
    type Error = GetUserError;

    async fn handle(&self, query: GetUserQuery) -> Result<Self::Response, Self::Error> {
        let user = self
            .user_repository
            .find_by_id(query.id)
            .await?
            .ok_or(GetUserError::NotAuthenticated)?;

        Ok(user)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetUserError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
