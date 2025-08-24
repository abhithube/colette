use colette_authentication::UserId;
use colette_common::RepositoryError;
use colette_core::tag::{TagError, TagId, TagRepository};

use crate::Handler;

#[derive(Debug, Clone)]
pub struct DeleteTagCommand {
    pub id: TagId,
    pub user_id: UserId,
}

pub struct DeleteTagHandler<TR: TagRepository> {
    tag_repository: TR,
}

impl<TR: TagRepository> DeleteTagHandler<TR> {
    pub fn new(tag_repository: TR) -> Self {
        Self { tag_repository }
    }
}

#[async_trait::async_trait]
impl<TR: TagRepository> Handler<DeleteTagCommand> for DeleteTagHandler<TR> {
    type Response = ();
    type Error = DeleteTagError;

    async fn handle(&self, cmd: DeleteTagCommand) -> Result<Self::Response, Self::Error> {
        self.tag_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => {
                    DeleteTagError::Tag(TagError::NotFound(cmd.id.as_inner()))
                }
                _ => DeleteTagError::Repository(e),
            })?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
