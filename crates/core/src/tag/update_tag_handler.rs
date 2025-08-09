use uuid::Uuid;

use super::{TagRepository, TagUpdateParams};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone, Default)]
pub struct UpdateTagCommand {
    pub id: Uuid,
    pub title: Option<String>,
    pub user_id: Uuid,
}

pub struct UpdateTagHandler {
    tag_repository: Box<dyn TagRepository>,
}

impl UpdateTagHandler {
    pub fn new(tag_repository: impl TagRepository) -> Self {
        Self {
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<UpdateTagCommand> for UpdateTagHandler {
    type Response = ();
    type Error = UpdateTagError;

    async fn handle(&self, cmd: UpdateTagCommand) -> Result<Self::Response, Self::Error> {
        let Some(tag) = self.tag_repository.find_by_id(cmd.id).await? else {
            return Err(UpdateTagError::NotFound(cmd.id));
        };
        if tag.user_id != cmd.user_id {
            return Err(UpdateTagError::Forbidden(cmd.id));
        }

        self.tag_repository
            .update(TagUpdateParams {
                id: cmd.id,
                title: cmd.title,
            })
            .await?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(Uuid),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
