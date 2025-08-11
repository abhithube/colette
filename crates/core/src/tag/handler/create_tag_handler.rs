use crate::{
    Handler,
    common::RepositoryError,
    tag::{TagId, TagInsertParams, TagRepository},
    user::UserId,
};

#[derive(Debug, Clone)]
pub struct CreateTagCommand {
    pub title: String,
    pub user_id: UserId,
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
                title: cmd.title.clone(),
                user_id: cmd.user_id,
            })
            .await
            .map_err(|e| match e {
                RepositoryError::Duplicate => CreateTagError::Conflict(cmd.title),
                _ => CreateTagError::Repository(e),
            })?;

        Ok(TagCreated { id })
    }
}

#[derive(Debug, Clone)]
pub struct TagCreated {
    pub id: TagId,
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTagError {
    #[error("tag already exists with title: {0}")]
    Conflict(String),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
