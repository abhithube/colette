use crate::{
    Handler,
    common::RepositoryError,
    tag::{TagError, TagId, TagRepository, TagUpdateParams},
    auth::UserId,
};

#[derive(Debug, Clone)]
pub struct UpdateTagCommand {
    pub id: TagId,
    pub title: Option<String>,
    pub user_id: UserId,
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
        let tag = self
            .tag_repository
            .find_by_id(cmd.id)
            .await?
            .ok_or_else(|| UpdateTagError::NotFound(cmd.id))?;
        tag.authorize(cmd.user_id)?;

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
    NotFound(TagId),

    #[error(transparent)]
    Core(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
