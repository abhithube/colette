use uuid::Uuid;

use super::TagRepository;
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct DeleteTagCommand {
    pub id: Uuid,
    pub user_id: Uuid,
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
        let Some(tag) = self.tag_repository.find_by_id(cmd.id).await? else {
            return Err(DeleteTagError::NotFound(cmd.id));
        };
        if tag.user_id != cmd.user_id {
            return Err(DeleteTagError::Forbidden(cmd.id));
        }

        self.tag_repository.delete_by_id(cmd.id).await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
