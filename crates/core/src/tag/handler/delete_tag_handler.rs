use crate::{
    Handler,
    common::RepositoryError,
    tag::{TagError, TagId, TagRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct DeleteTagCommand {
    pub id: TagId,
    pub user_id: UserId,
}

pub struct DeleteTagHandler {
    tag_repository: Box<dyn TagRepository>,
}

impl DeleteTagHandler {
    pub fn new(tag_repository: impl TagRepository) -> Self {
        Self {
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<DeleteTagCommand> for DeleteTagHandler {
    type Response = ();
    type Error = DeleteTagError;

    async fn handle(&self, cmd: DeleteTagCommand) -> Result<Self::Response, Self::Error> {
        let tag = self
            .tag_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| DeleteTagError::NotFound(cmd.id))?;
        tag.authorize(cmd.user_id)?;

        self.tag_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(TagId),

    #[error(transparent)]
    Core(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
