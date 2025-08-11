use crate::{
    Handler, User,
    common::RepositoryError,
    user::{UserId, UserRepository},
};

#[derive(Debug, Clone)]
pub struct GetUserQuery {
    pub id: UserId,
}

pub struct GetUserHandler {
    user_repository: Box<dyn UserRepository>,
}

impl GetUserHandler {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<GetUserQuery> for GetUserHandler {
    type Response = User;
    type Error = GetUserError;

    async fn handle(&self, query: GetUserQuery) -> Result<Self::Response, Self::Error> {
        let user = self
            .user_repository
            .find_by_id(query.id)
            .await?
            .ok_or_else(|| GetUserError::NotAuthenticated)?;

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
