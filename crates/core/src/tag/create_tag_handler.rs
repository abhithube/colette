use uuid::Uuid;

use super::{TagInsertParams, TagRepository};
use crate::{Handler, RepositoryError};

#[derive(Debug, Clone)]
pub struct CreateTagCommand {
    pub title: String,
    pub user_id: Uuid,
}

pub struct CreateTagHandler {
    tag_repository: Box<dyn TagRepository>,
}

impl CreateTagHandler {
    pub fn new(tag_repository: impl TagRepository) -> Self {
        Self {
            tag_repository: Box::new(tag_repository),
        }
    }
}

#[async_trait::async_trait]
impl Handler<CreateTagCommand> for CreateTagHandler {
    type Response = TagCreated;
    type Error = CreateTagError;

    async fn handle(&self, cmd: CreateTagCommand) -> Result<Self::Response, Self::Error> {
        let id = self
            .tag_repository
            .insert(TagInsertParams {
                title: cmd.title,
                user_id: cmd.user_id,
            })
            .await?;

        Ok(TagCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct TagCreated {
    pub id: Uuid,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTagError {
    #[error("tag not found with ID: {0}")]
    NotFound(Uuid),

    #[error("not authorized to access tag with ID: {0}")]
    Forbidden(Uuid),

    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
