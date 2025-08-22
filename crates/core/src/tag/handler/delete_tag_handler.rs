use crate::{
    Handler,
    auth::UserId,
    common::RepositoryError,
    tag::{TagError, TagId, TagRepository},
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
        self.tag_repository
            .delete_by_id(cmd.id, cmd.user_id)
            .await
            .map_err(|e| match e {
                RepositoryError::NotFound => DeleteTagError::Tag(TagError::NotFound(cmd.id)),
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
