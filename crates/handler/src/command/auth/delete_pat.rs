use colette_core::{
    auth::{PatId, UserError, UserId, UserRepository},
    common::RepositoryError,
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeletePatCommand {
    pub id: PatId,
    pub user_id: UserId,
}

pub struct DeletePatHandler<UR: UserRepository> {
    user_repository: UR,
}

impl<UR: UserRepository> DeletePatHandler<UR> {
    pub fn new(user_repository: UR) -> Self {
        Self { user_repository }
    }
}

#[async_trait::async_trait]
impl<UR: UserRepository> Handler<DeletePatCommand> for DeletePatHandler<UR> {
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
