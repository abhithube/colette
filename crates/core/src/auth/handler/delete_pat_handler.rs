use crate::{
    Handler,
    auth::{PatId, UserError, UserId, UserRepository},
    common::RepositoryError,
};

#[derive(Debug, Clone)]
pub struct DeletePatCommand {
    pub id: PatId,
    pub user_id: UserId,
}

pub struct DeletePatHandler {
    user_repository: Box<dyn UserRepository>,
}

impl DeletePatHandler {
    pub fn new(user_repository: impl UserRepository) -> Self {
        Self {
            user_repository: Box::new(user_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeletePatCommand> for DeletePatHandler {
    type Response = ();
    type Error = DeletePatError;

    async fn handle(&self, cmd: DeletePatCommand) -> Result<Self::Response, Self::Error> {
        let mut user = self
            .user_repository
            .find_by_id(cmd.user_id)
            .await?
            .ok_or(DeletePatError::NotAuthenticated)?;

        user.remove_personal_access_token(cmd.id)?;

        self.user_repository.save(&user).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeletePatError {
    #[error("user not authenticated")]
    NotAuthenticated,

    #[error(transparent)]
    User(#[from] UserError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
