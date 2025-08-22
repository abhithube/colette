use crate::{
    Handler, Tag,
    auth::UserId,
    common::RepositoryError,
    tag::{TagError, TagRepository, TagTitle},
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
    type Response = Tag;
    type Error = CreateTagError;

    async fn handle(&self, cmd: CreateTagCommand) -> Result<Self::Response, Self::Error> {
        let title = TagTitle::new(cmd.title)?;

        let tag = Tag::new(title.clone(), cmd.user_id);

        self.tag_repository.save(&tag).await.map_err(|e| match e {
            RepositoryError::Duplicate => CreateTagError::Tag(TagError::Conflict(title)),
            _ => CreateTagError::Repository(e),
        })?;

        Ok(tag)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateTagError {
    #[error(transparent)]
    Tag(#[from] TagError),

    #[error(transparent)]
    Repository(#[from] RepositoryError),
}
