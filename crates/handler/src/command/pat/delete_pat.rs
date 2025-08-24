use colette_core::{
    auth::UserId,
    common::RepositoryError,
    pat::{PatError, PatId, PatRepository},
};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeletePatCommand {
    pub id: PatId,
    pub user_id: UserId,
}

pub struct DeletePatHandler<PR: PatRepository> {
    pat_repository: PR,
}

impl<PR: PatRepository> DeletePatHandler<PR> {
    pub fn new(pat_repository: PR) -> Self {
        Self { pat_repository }
    }
}

#[async_trait::async_trait]
impl<PR: PatRepository> Handler<DeletePatCommand> for DeletePatHandler<PR> {
    type Response = ();
    type Error = DeletePatError;

    async fn handle(&self, cmd: DeletePatCommand) -> Result<Self::Response, Self::Error> {
        self.pat_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeletePatError::Pat(PatError::NotFound(cmd.id.as_inner()))
                }
                _ => DeletePatError::Repository(e),
            })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeletePatError {
    #[error(transparent)]
    Pat(#[from] PatError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
