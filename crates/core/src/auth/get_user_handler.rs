use uuid::Uuid;

use crate::{Handler, RepositoryError, User, user::UserRepository};

#[derive(Debug, Clone)]
pub struct GetUserQuery {
    pub id: Uuid,
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
        let Some(user) = self.user_repository.find_by_id(query.id).await? else {
            return Err(GetUserError::NotAuthenticated);
        };

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
